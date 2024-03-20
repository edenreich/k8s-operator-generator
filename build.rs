use askama::Template;
use inflector::Inflector;
use openapiv3::{OpenAPI, Schema};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{DirBuilder, File, OpenOptions},
    io::{BufRead, BufReader, Error, Read, Write},
    process::Command,
    vec,
};

const CONTROLLERS_DIR: &str = "src/controllers";
const TYPES_DIR: &str = "src/types";
const RBAC_DIR: &str = "manifests/rbac";
const LIB_FILEPATH: &str = "src/lib.rs";
const API_GROUP: &str = "example.com";
const API_VERSION: &str = "v1";
const RESOURCE_REF: &str = "uuid";

fn main() {
    let input = "openapi.yaml";

    // Read the OpenAPI specification from the YAML file
    let mut file = File::open(input).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    // Parse the OpenAPI specification
    let openapi: OpenAPI = serde_yaml::from_str(&contents).expect("Unable to parse OpenAPI spec");

    // Generate lib.rs
    generate_lib();

    let components = openapi
        .components
        .clone()
        .expect("No components in OpenAPI spec");

    // Create the types directory if it doesn't exist
    DirBuilder::new()
        .recursive(true)
        .create(TYPES_DIR)
        .expect("Unable to create types directory");

    // Generate types for each schema in the components
    for (name, schema_item) in components.schemas.iter() {
        let controller_name = format!("{}", name.to_plural());
        match schema_item {
            openapiv3::ReferenceOr::Reference { reference } => {
                // Handle the case where the item is a reference.
                // You might need to look up the actual schema using the reference.
            }
            openapiv3::ReferenceOr::Item(item) => {
                // Here, `item` is an `openapiv3::Schema`.
                // You can generate a type for it and add it to the mod file.
                generate_type(&name, API_VERSION, &item);
                add_type_to_modfile(&name).expect("Unable to add type to mod file");
                generate_controller(&controller_name, item);
                add_controller_to_modfile(controller_name.as_str())
                    .expect("Unable to add controller to mod file");
                upsert_line_to_file(
                    ".openapi-generator-ignore".to_string(),
                    format!("src/controllers/{}.rs", controller_name.to_lowercase()),
                )
                .expect("Unable to add controller to .openapi-generator-ignore");
            }
        }
    }

    // Generate the RBAC for the operator
    let mut resources = vec![];
    for (schema_name, _) in components.schemas.iter() {
        resources.push(schema_name.to_lowercase().to_plural());
    }

    generate_role_file(resources.to_owned());
    generate_cluster_role_file(resources.to_owned());
    generate_service_account_file();
    generate_role_binding_file_content();
    generate_cluster_role_binding_file_content();
    generate_operator_deployment_file();
    generate_crdgen_file(resources);
    generate_examples(components.examples.into_iter().collect());
}

struct Field {
    pub_name: String,
    field_type: String,
}

#[derive(Template)]
#[template(path = "type.jinja")]
struct TypeTemplate {
    tag_name: String,
    type_name: String,
    api_version: String,
    group_name: String,
    fields: Vec<Field>,
    reference_id: String,
}

fn get_fields_for_type(schema: &Schema) -> Vec<Field> {
    let mut fields = vec![];
    if let openapiv3::SchemaKind::Type(openapiv3::Type::Object(object)) = &schema.schema_kind {
        for (field_name, field_schema) in &object.properties {
            match field_schema {
                openapiv3::ReferenceOr::Reference { .. } => {
                    // Handle references here if needed
                }
                openapiv3::ReferenceOr::Item(item) => {
                    let base_field_type = match &item.schema_kind {
                        openapiv3::SchemaKind::Type(openapiv3::Type::String(_)) => "String",
                        openapiv3::SchemaKind::Type(openapiv3::Type::Integer(_)) => "i32",
                        openapiv3::SchemaKind::Type(openapiv3::Type::Number(_)) => "f64",
                        openapiv3::SchemaKind::Type(openapiv3::Type::Boolean(_)) => "bool",
                        openapiv3::SchemaKind::Type(openapiv3::Type::Array(_)) => "Vec<_>",
                        // Add more cases here for other types as needed
                        _ => continue, // Skip unknown types
                    };
                    let field_type = if object.required.contains(field_name) {
                        base_field_type.to_string()
                    } else {
                        format!("Option<{}>", base_field_type)
                    };
                    fields.push(Field {
                        pub_name: field_name.clone(),
                        field_type,
                    });
                }
            }
        }
    }
    fields.retain(|field| field.pub_name != RESOURCE_REF);
    fields
}

fn generate_type(name: &str, api_version: &str, schema: &Schema) {
    if get_ignored_files().contains(&format!("src/types/{}.rs", name.to_lowercase())) {
        return;
    }

    let fields = get_fields_for_type(schema);

    let tag_name = name.to_string().to_lowercase().to_plural();
    let arg_name = name.to_lowercase();
    let type_name = uppercase_first_letter(name);
    let arg_name_clone = arg_name.clone();

    let content: String = TypeTemplate {
        tag_name,
        type_name,
        api_version: api_version.to_string(),
        group_name: API_GROUP.to_string(),
        fields,
        reference_id: RESOURCE_REF.to_string(),
    }
    .render()
    .unwrap();

    let file_path = format!("src/types/{}.rs", arg_name_clone);
    write_to_file(file_path.to_string(), content);
    format_file(file_path.to_string());
}
#[derive(Template)]
#[template(path = "lib.jinja")]
struct LibTemplate {}

fn generate_lib() {
    if get_ignored_files().contains(&"src/lib.rs".to_string()) {
        return;
    }

    let content: String = LibTemplate {}.render().unwrap();
    let file_path = LIB_FILEPATH;
    write_to_file(file_path.to_string(), content);
    format_file(file_path.to_string());
}

fn add_type_to_modfile(type_name: &str) -> Result<(), Error> {
    let file_path = format!("{}/mod.rs", TYPES_DIR);
    upsert_line_to_file(file_path, format!("pub mod {};", type_name.to_lowercase()))
}

fn add_controller_to_modfile(controller_name: &str) -> Result<(), Error> {
    let file_path = format!("{}/mod.rs", CONTROLLERS_DIR);
    upsert_line_to_file(
        file_path,
        format!("pub mod {};", controller_name.to_lowercase()),
    )
}

#[derive(Template)]
#[template(path = "controller.jinja")]
struct ControllerTemplate<'a> {
    tag_name: &'a str,
    arg_name: &'a str,
    type_name: &'a str,
    fields: Vec<Field>,
    reference_id: &'a str,
}

fn generate_controller(name: &str, schema: &Schema) {
    if get_ignored_files().contains(&format!("{}/{}.rs", CONTROLLERS_DIR, name.to_lowercase())) {
        return;
    }

    let fields = get_fields_for_type(schema);
    let name_singular = name.to_singular();
    let content: String = ControllerTemplate {
        tag_name: &name.to_lowercase(),
        arg_name: name_singular.clone().to_lowercase().as_str(),
        type_name: &uppercase_first_letter(name_singular.clone().as_str()),
        fields: fields,
        reference_id: RESOURCE_REF,
    }
    .render()
    .unwrap();
    let file_path = format!("{}/{}.rs", CONTROLLERS_DIR, name.to_lowercase());
    write_to_file(file_path.to_owned(), content);
    format_file(file_path)
}

#[derive(Template)]
#[template(path = "crdgen.jinja")]
struct CrdGenTemplate {
    resources: Vec<String>,
}

fn generate_crdgen_file(resources: Vec<String>) {
    if get_ignored_files().contains(&"src/crdgen.rs".to_string()) {
        return;
    }

    let resources = resources
        .into_iter()
        .map(|resource| resource.to_singular())
        .collect();
    let template = CrdGenTemplate { resources };
    let content = template.render().unwrap();
    write_to_file("src/crdgen.rs".to_string(), content);
    format_file("src/crdgen.rs".to_string());
}

fn get_ignored_files() -> Vec<String> {
    let ignore_file =
        std::fs::File::open(".openapi-generator-ignore").expect("Unable to open file");
    let reader = BufReader::new(ignore_file);
    reader.lines().filter_map(Result::ok).collect()
}

fn empty_file(file_path: String) -> Result<(), Error> {
    match File::create(&file_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn write_to_file(file_path: String, file_content: String) {
    if get_ignored_files().contains(&file_path) {
        return;
    }

    std::fs::write(file_path, file_content + "\n").expect("Unable to write file");
}

fn upsert_line_to_file(file_path: String, line: String) -> Result<(), Error> {
    if get_ignored_files().contains(&file_path) {
        return Ok(());
    }

    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    let exists = reader.lines().any(|l| l.unwrap() == line);

    if !exists {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&file_path)?;
        if let Err(e) = writeln!(file, "{}", line) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
    Ok(())
}

fn format_file(file_path: String) {
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let output = Command::new("rustfmt")
        .arg(file_path)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        eprintln!(
            "rustfmt failed with output:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn uppercase_first_letter(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

struct RoleTemplateIdentifiers {
    api_group: String,
    resources: Vec<String>,
}

#[derive(Template)]
#[template(path = "manifest_rbac_role.jinja")]
struct RoleTemplate {
    identifiers: RoleTemplateIdentifiers,
}

fn generate_role_file(resources: Vec<String>) {
    if get_ignored_files().contains(&format!("{}/role.yaml", RBAC_DIR)) {
        return;
    }

    let content = RoleTemplate {
        identifiers: RoleTemplateIdentifiers {
            api_group: API_GROUP.to_string(),
            resources: resources,
        },
    }
    .render()
    .unwrap();
    write_to_file(format!("{}/role.yaml", RBAC_DIR), content);
}

struct ClusterRoleTemplateIdentifiers {
    api_group: String,
    resources: Vec<String>,
}

#[derive(Template)]
#[template(path = "manifest_rbac_role.jinja")]
struct ClusterRoleTemplate {
    identifiers: ClusterRoleTemplateIdentifiers,
}

fn generate_cluster_role_file(resources: Vec<String>) {
    if get_ignored_files().contains(&format!("{}/clusterrole.yaml", RBAC_DIR)) {
        return;
    }

    let content = ClusterRoleTemplate {
        identifiers: ClusterRoleTemplateIdentifiers {
            api_group: API_GROUP.to_string(),
            resources: resources,
        },
    }
    .render()
    .unwrap();
    write_to_file(format!("{}/clusterrole.yaml", RBAC_DIR), content);
}

#[derive(Template)]
#[template(path = "manifest_rbac_service_account.jinja")]
struct ServiceAccountTemplate {}

fn generate_service_account_file() {
    if get_ignored_files().contains(&format!("{}/serviceaccount.yaml", RBAC_DIR)) {
        return;
    }

    let content = ServiceAccountTemplate {}.render().unwrap();
    write_to_file(format!("{}/serviceaccount.yaml", RBAC_DIR), content);
}

#[derive(Template)]
#[template(path = "manifest_rbac_role_binding.jinja")]
struct RoleBindingTemplate {}

fn generate_role_binding_file_content() {
    if get_ignored_files().contains(&format!("{}/rolebinding.yaml", RBAC_DIR)) {
        return;
    }

    let content = RoleBindingTemplate {}.render().unwrap();
    write_to_file(format!("{}/rolebinding.yaml", RBAC_DIR), content);
}

#[derive(Template)]
#[template(path = "manifest_rbac_cluster_role_binding.jinja")]
struct ClusterRoleBindingTemplate {}

fn generate_cluster_role_binding_file_content() {
    if get_ignored_files().contains(&format!("{}/clusterrolebinding.yaml", RBAC_DIR)) {
        return;
    }

    let content = ClusterRoleBindingTemplate {}.render().unwrap();
    write_to_file(format!("{}/clusterrolebinding.yaml", RBAC_DIR), content);
}

#[derive(Template)]
#[template(path = "manifest_operator_deployment.jinja")]
struct OperatorDeploymentTemplate {}

fn generate_operator_deployment_file() {
    if get_ignored_files().contains(&"manifests/operator/deployment.yaml".to_string()) {
        return;
    }

    let content = OperatorDeploymentTemplate {}.render().unwrap();
    write_to_file("manifests/operator/deployment.yaml".to_string(), content);
}

#[derive(Template, Deserialize, Serialize)]
#[template(path = "manifest_example.jinja")]
struct ExampleTemplate {
    api_version: String,
    kind: String,
    metadata: Metadata,
    spec: String,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
    name: String,
}

fn generate_examples(
    examples: std::collections::HashMap<String, openapiv3::ReferenceOr<openapiv3::Example>>,
) {
    let examples_map: std::collections::HashMap<String, openapiv3::Example> = examples
        .into_iter()
        .filter_map(|(k, v)| match v {
            openapiv3::ReferenceOr::Item(item) => Some((k, item)),
            openapiv3::ReferenceOr::Reference { .. } => None,
        })
        .collect();
    for (name, example) in &examples_map {
        generate_manifest_from_example(&name, example);
    }
}

fn generate_manifest_from_example(name: &str, example: &openapiv3::Example) {
    let spec = match &example.value {
        Some(Value::Object(map)) => {
            let mut map = map.clone();
            map.remove(RESOURCE_REF);
            let json_value = Value::Object(map);
            let yaml_value = json!(json_value);
            let mut yaml_string =
                serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| String::from("{}"));
            yaml_string = yaml_string
                .lines()
                .map(|line| format!("  {}", line))
                .collect::<Vec<_>>()
                .join("\n");
            yaml_string
        }
        _ => {
            eprintln!("Example value is not an object");
            return;
        }
    };

    let template = ExampleTemplate {
        api_version: API_VERSION.to_string(),
        kind: uppercase_first_letter(name),
        metadata: Metadata {
            name: "example".to_string(),
        },
        spec,
    };

    match template.render() {
        Ok(content) => {
            write_to_file(
                format!("manifests/examples/{}.yaml", name.to_lowercase()),
                content,
            );
        }
        Err(e) => {
            eprintln!("Failed to render template: {}", e);
        }
    }
}
