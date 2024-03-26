use askama::Template;
use inflector::Inflector;
use log::{error, warn};
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    collections::{HashMap, HashSet},
    fs::{DirBuilder, File, OpenOptions},
    io::{BufRead, BufReader, Error, Write},
    process::Command,
    vec,
};

const OPENAPI_FILE: &str = "openapi.yaml";
const CONTROLLERS_DIR: &str = "src/controllers";
const TYPES_DIR: &str = "src/types";
const RBAC_DIR: &str = "manifests/rbac";
const EXAMPLES_DIR: &str = "manifests/examples";
const LIB_FILEPATH: &str = "src/lib.rs";
const CRDGEN_FILEPATH: &str = "crdgen/main.rs";

fn main() {
    env_logger::init();

    let openapi = read_and_parse_openapi_spec(OPENAPI_FILE);
    let (
        kubernetes_operator_group,
        kubernetes_operator_version,
        kubernetes_operator_resource_ref,
        kubernetes_operator_include_tags,
    ) = extract_openapi_info(&openapi);
    let components = openapi
        .components
        .clone()
        .expect("No components in OpenAPI spec");
    let paths: openapiv3::Paths = openapi.paths.clone();

    create_directory_if_not_exists(RBAC_DIR);
    create_directory_if_not_exists(EXAMPLES_DIR);
    create_directory_if_not_exists(TYPES_DIR);
    create_directory_if_not_exists(CONTROLLERS_DIR);

    generate_lib();
    generate_all_files(
        paths,
        kubernetes_operator_group,
        kubernetes_operator_version,
        kubernetes_operator_resource_ref,
        kubernetes_operator_include_tags,
        components,
    );
}

fn read_and_parse_openapi_spec(file_path: &str) -> OpenAPI {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Unable to parse OpenAPI spec")
}

fn extract_openapi_info(openapi: &OpenAPI) -> (String, String, String, Vec<String>) {
    let kubernetes_operator_group = extract_extension(openapi, "x-kubernetes-operator-group");
    let kubernetes_operator_version = extract_extension(openapi, "x-kubernetes-operator-version");
    let kubernetes_operator_resource_ref =
        extract_extension(openapi, "x-kubernetes-operator-resource-ref");
    let kubernetes_operator_include_tags =
        extract_extension_array(openapi, "x-kubernetes-operator-include-tags");
    (
        kubernetes_operator_group,
        kubernetes_operator_version,
        kubernetes_operator_resource_ref,
        kubernetes_operator_include_tags,
    )
}

fn extract_extension(openapi: &OpenAPI, key: &str) -> String {
    openapi
        .info
        .extensions
        .get(key)
        .expect(&format!("No {} in OpenAPI spec", key))
        .as_str()
        .expect(&format!("{} is not a string", key))
        .to_string()
}

fn extract_extension_array(openapi: &OpenAPI, key: &str) -> Vec<String> {
    openapi
        .info
        .extensions
        .get(key)
        .expect(&format!("No {} in OpenAPI spec", key))
        .as_array()
        .expect(&format!("{} is not an array", key))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn create_directory_if_not_exists(dir: &str) {
    DirBuilder::new()
        .recursive(true)
        .create(dir)
        .expect(&format!("Unable to create {} directory", dir));
}

fn generate_all_files(
    paths: openapiv3::Paths,
    kubernetes_operator_group: String,
    kubernetes_operator_version: String,
    kubernetes_operator_resource_ref: String,
    kubernetes_operator_include_tags: Vec<String>,
    components: openapiv3::Components,
) {
    let schemas: HashMap<String, Schema> = components
        .schemas
        .iter()
        .filter_map(|(name, schema)| {
            match schema {
                openapiv3::ReferenceOr::Item(schema) => Some((name.clone(), schema.clone())),
                openapiv3::ReferenceOr::Reference { .. } => None, // Ignore references for now
            }
        })
        .collect();

    let mut schema_names = vec![];
    for (schema_name, _) in components.schemas.iter() {
        schema_names.push(schema_name.to_lowercase().to_plural());
    }
    generate_types(schemas.clone(), &kubernetes_operator_resource_ref);

    generate_controllers(
        schemas.clone(),
        paths.clone(),
        kubernetes_operator_include_tags,
        kubernetes_operator_resource_ref.clone(),
    );

    generate_rbac_files(schema_names.clone(), &kubernetes_operator_group);
    generate_crdgen_file(schema_names.clone());
    generate_examples(
        components.examples.into_iter().collect(),
        &kubernetes_operator_group,
        &kubernetes_operator_version,
        &kubernetes_operator_resource_ref.clone(),
    );
}

fn generate_rbac_files(resources: Vec<String>, kubernetes_operator_group: &str) {
    generate_role_file(resources.clone(), kubernetes_operator_group);
    generate_cluster_role_file(resources.clone(), kubernetes_operator_group);
    generate_service_account_file();
    generate_role_binding_file_content();
    generate_cluster_role_binding_file_content();
    generate_operator_deployment_file();
}

struct ControllerAttributes {
    operation_id: String,
    http_method: String,
    action_summary: String,
}

fn get_controller_attributes_for_operation(
    operation: &openapiv3::Operation,
    http_method: &str,
    include_tags: &[String],
) -> Option<(String, ControllerAttributes)> {
    // Check if the operation has any of the included tags
    let tag: &String = operation
        .tags
        .iter()
        .find(|tag| include_tags.contains(tag))?;

    // Get the operation ID, which will be used as the method name
    let operation_id = operation.operation_id.as_ref()?;

    // Initialize vectors for request body schema and response schemas
    let mut request_body_schemas = HashSet::new();

    // Get the request body schema
    if let Some(ReferenceOr::Item(item)) = &operation.request_body {
        if let Some(content) = item.content.get("application/json") {
            if let Some(ReferenceOr::Reference { reference }) = content.schema.as_ref() {
                // Parse the reference to get the schema name
                let schema_name = reference
                    .split('/')
                    .last()
                    .unwrap_or("UnnamedSchema")
                    .to_string();
                // Add the schema name to the HashSet
                request_body_schemas.insert(schema_name);
            }
        }
    }

    let mut response_schemas = Vec::new();

    // Get the responses schema
    if let Some(ReferenceOr::Item(item)) = operation.responses.default.as_ref() {
        if let Some(content) = item.content.get("application/json") {
            if let Some(ReferenceOr::Reference { reference }) = content.schema.as_ref() {
                // Parse the reference to get the schema name
                let schema_name = reference
                    .split('/')
                    .last()
                    .unwrap_or("UnnamedSchema")
                    .to_string();
                response_schemas.push(schema_name);
            }
        }
    }

    // Create the controller attributes
    let attributes = ControllerAttributes {
        operation_id: operation_id.to_string().to_snake_case(),
        http_method: http_method.to_string(),
        action_summary: operation.summary.clone().unwrap_or_default().to_lowercase(),
    };

    Some((tag.clone(), attributes))
}

fn generate_controllers(
    schemas: HashMap<String, Schema>,
    paths: openapiv3::Paths,
    include_tags: Vec<String>,
    kubernetes_operator_resource_ref: String,
) {
    let mut controllers: HashMap<String, Vec<ControllerAttributes>> = HashMap::new();

    for (_path, path_item) in paths {
        match path_item {
            ReferenceOr::Reference { reference: _ } => {}
            ReferenceOr::Item(item) => {
                let operations = vec![
                    ("get", &item.get),
                    ("post", &item.post),
                    ("delete", &item.delete),
                    ("put", &item.put),
                ];

                for (method, operation) in operations {
                    if let Some(operation) = operation {
                        if operation
                            .tags
                            .iter()
                            .any(|tag: &String| include_tags.contains(tag))
                        {
                            if let Some((tag, controller)) = get_controller_attributes_for_operation(
                                operation,
                                method,
                                &include_tags,
                            ) {
                                controllers
                                    .entry(tag.clone())
                                    .or_insert_with(Vec::new)
                                    .push(controller);
                            }
                        }
                    }
                }
            }
        }
    }

    for (tag, controller_attributes) in controllers {
        generate_controller(
            schemas.clone(),
            tag.clone(),
            controller_attributes,
            kubernetes_operator_resource_ref.clone(),
        );

        match upsert_line_to_file(
            ".openapi-generator-ignore",
            format!("{}/{}.rs", CONTROLLERS_DIR, tag.to_lowercase()).as_str(),
        ) {
            Ok(_) => (),
            Err(e) => error!(
                "Failed to add controller to .openapi-generator-ignore file: {:?}",
                e
            ),
        }
    }
}

#[derive(Template)]
#[template(path = "controller.jinja")]
struct ControllerTemplate<'a> {
    tag: String,
    arg_name: String,
    kind_struct: String,
    controllers: Vec<&'a ControllerAttributes>,
    dto_fields: Vec<Field>,
    resource_remote_ref: String,
    has_create_action: bool,
    has_update_action: bool,
    has_delete_action: bool,
}

#[derive(Template)]
#[template(path = "controller_action_delete.jinja")]
struct ControllerActionDeleteTemplate<'a> {
    arg_name: String,
    kind_struct: String,
    controllers: Vec<&'a ControllerAttributes>,
    resource_remote_ref: String,
}

#[derive(Template)]
#[template(path = "controller_action_put.jinja")]
struct ControllerActionPutTemplate<'a> {
    arg_name: String,
    kind_struct: String,
    controllers: Vec<&'a ControllerAttributes>,
    resource_remote_ref: String,
}

#[derive(Template)]
#[template(path = "controller_action_post.jinja")]
struct ControllerActionPostTemplate<'a> {
    arg_name: String,
    kind_struct: String,
    controllers: Vec<&'a ControllerAttributes>,
    resource_remote_ref: String,
}

struct Field {
    pub_name: String,
    field_type: String,
}

fn generate_controller(
    schemas: HashMap<String, Schema>,
    tag: String,
    controller_attributes: Vec<ControllerAttributes>,
    resource_remote_ref: String,
) {
    if get_ignored_files().contains(&format!("{}/{}.rs", CONTROLLERS_DIR, tag.to_lowercase())) {
        return;
    }

    let has_create_action = controller_attributes
        .iter()
        .any(|controller| controller.http_method == "post");

    let has_update_action = controller_attributes
        .iter()
        .any(|controller| controller.http_method == "put");

    let has_delete_action = controller_attributes
        .iter()
        .any(|controller| controller.http_method == "delete");

    let type_name = uppercase_first_letter(&tag.to_singular());

    let fields = match get_fields_for_type(&schemas, &type_name, &resource_remote_ref) {
        Ok(fields) => fields,
        Err(e) => {
            error!("Failed to get fields for type: {:?}", e);
            return;
        }
    };

    let mut content: String = ControllerTemplate {
        tag: tag.to_lowercase(),
        arg_name: tag.to_lowercase().to_singular(),
        kind_struct: type_name.clone(),
        controllers: controller_attributes.iter().collect(),
        dto_fields: fields,
        resource_remote_ref: resource_remote_ref.clone(),
        has_create_action,
        has_update_action,
        has_delete_action,
    }
    .render()
    .unwrap();

    let content_action_delete: String = ControllerActionDeleteTemplate {
        arg_name: tag.to_lowercase().to_singular(),
        kind_struct: type_name.clone(),
        controllers: controller_attributes.iter().collect(),
        resource_remote_ref: resource_remote_ref.clone(),
    }
    .render()
    .unwrap();

    let content_action_put: String = ControllerActionPutTemplate {
        arg_name: tag.to_lowercase().to_singular(),
        kind_struct: type_name.clone(),
        controllers: controller_attributes.iter().collect(),
        resource_remote_ref: resource_remote_ref.clone(),
    }
    .render()
    .unwrap();

    let content_action_post: String = ControllerActionPostTemplate {
        arg_name: tag.to_lowercase().to_singular(),
        kind_struct: type_name,
        controllers: controller_attributes.iter().collect(),
        resource_remote_ref: resource_remote_ref.clone(),
    }
    .render()
    .unwrap();

    content.push_str(&content_action_delete);
    content.push_str(&content_action_put);
    content.push_str(&content_action_post);

    let file_path: String = format!("{}/{}.rs", CONTROLLERS_DIR, tag.to_lowercase());
    write_to_file(file_path.to_string(), content);
    format_file(file_path.to_string());
    add_controller_to_modfile(&tag.to_lowercase()).expect("Failed to add controller to mod file");
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

fn get_fields_for_type(
    schemas: &HashMap<String, Schema>,
    schema_name: &str,
    operator_resource_ref: &str,
) -> Result<Vec<Field>, Box<dyn std::error::Error>> {
    let schema = schemas
        .get(schema_name)
        .ok_or_else(|| format!("Schema {} not found", schema_name))?;

    let object = match &schema.schema_kind {
        SchemaKind::Type(Type::Object(object)) => object,
        _ => return Ok(vec![]), // Early return for non-object types
    };

    let fields = object
        .properties
        .iter()
        .filter_map(|(field_name, field_schema)| {
            let item = match field_schema {
                ReferenceOr::Item(item) => item,
                _ => return None, // Skip non-item types
            };

            let base_field_type = match &item.schema_kind {
                SchemaKind::Type(Type::String(_)) => "String",
                SchemaKind::Type(Type::Integer(_)) => "i32",
                SchemaKind::Type(Type::Number(_)) => "f64",
                SchemaKind::Type(Type::Boolean(_)) => "bool",
                SchemaKind::Type(Type::Array(_)) => "Vec<_>",
                // Add more cases here for other types as needed
                _ => return None, // Skip unknown types
            };

            let field_type = if object.required.contains(field_name) {
                base_field_type.to_string()
            } else {
                format!("Option<{}>", base_field_type)
            };

            Some(Field {
                pub_name: field_name.clone(),
                field_type,
            })
        })
        .filter(|field| field.pub_name != operator_resource_ref)
        .collect();

    Ok(fields)
}

fn generate_types(schemas: HashMap<String, Schema>, operator_resource_ref: &str) {
    for name in schemas.keys() {
        generate_type(
            schemas.clone(),
            &name,
            "example.com",
            "v1",
            operator_resource_ref,
        );
        match add_type_to_modfile(&name) {
            Ok(_) => (),
            Err(e) => error!("Failed to add type to mod file: {:?}", e),
        }
    }
}

fn generate_type(
    schemas: HashMap<String, Schema>,
    name: &str,
    operator_group: &str,
    operator_version: &str,
    operator_resource_ref: &str,
) {
    if get_ignored_files().contains(&format!("{}/{}.rs", TYPES_DIR, name.to_lowercase())) {
        return;
    }

    let fields = match get_fields_for_type(&schemas, name, operator_resource_ref) {
        Ok(fields) => fields,
        Err(e) => {
            error!("Failed to get fields for type: {:?}", e);
            return;
        }
    };

    let tag_name = name.to_string().to_lowercase().to_plural();
    let arg_name = name.to_lowercase();
    let type_name = uppercase_first_letter(name);
    let arg_name_clone = arg_name.clone();

    let content: String = TypeTemplate {
        tag_name,
        type_name,
        api_version: operator_version.to_string(),
        group_name: operator_group.to_string(),
        fields,
        reference_id: operator_resource_ref.to_string(),
    }
    .render()
    .unwrap();

    let file_path = format!("{}/{}.rs", TYPES_DIR, arg_name_clone);
    write_to_file(file_path.to_string(), content);
    format_file(file_path.to_string());
}
#[derive(Template)]
#[template(path = "lib.jinja")]
struct LibTemplate {}

fn generate_lib() {
    if get_ignored_files().contains(&LIB_FILEPATH.to_string()) {
        return;
    }

    let content: String = LibTemplate {}.render().unwrap();
    let file_path = LIB_FILEPATH;
    write_to_file(file_path.to_string(), content);
    format_file(file_path.to_string());
}

fn add_type_to_modfile(type_name: &str) -> Result<(), Error> {
    let file_path = format!("{}/mod.rs", TYPES_DIR);
    match upsert_line_to_file(
        file_path.as_str(),
        format!("pub mod {};", type_name.to_lowercase()).as_str(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn add_controller_to_modfile(controller_name: &str) -> Result<(), Error> {
    let file_path = format!("{}/mod.rs", CONTROLLERS_DIR);
    match upsert_line_to_file(
        file_path.as_str(),
        format!("pub mod {};", controller_name.to_lowercase()).as_str(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[derive(Template)]
#[template(path = "crdgen.jinja")]
struct CrdGenTemplate {
    resources: Vec<String>,
}

fn generate_crdgen_file(resources: Vec<String>) {
    if get_ignored_files().contains(&CRDGEN_FILEPATH.to_string()) {
        return;
    }

    let resources = resources
        .into_iter()
        .map(|resource| resource.to_singular())
        .collect();
    let template = CrdGenTemplate { resources };
    let content = template.render().unwrap();
    write_to_file(CRDGEN_FILEPATH.to_string(), content);
    format_file(CRDGEN_FILEPATH.to_string());
}

fn get_ignored_files() -> Vec<String> {
    let ignore_file =
        std::fs::File::open(".openapi-generator-ignore").expect("Unable to open file");
    let reader = BufReader::new(ignore_file);
    reader.lines().filter_map(Result::ok).collect()
}

fn write_to_file(file_path: String, file_content: String) {
    if get_ignored_files().contains(&file_path) {
        return;
    }

    std::fs::write(file_path, file_content + "\n").expect("Unable to write file");
}

fn upsert_line_to_file(file_path: &str, line: &str) -> Result<(), Error> {
    if get_ignored_files().contains(&file_path.to_string()) {
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
            error!("Couldn't write to file: {}", e);
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
        error!(
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

fn generate_role_file(resources: Vec<String>, api_group: &str) {
    if get_ignored_files().contains(&format!("{}/role.yaml", RBAC_DIR)) {
        return;
    }

    let content = RoleTemplate {
        identifiers: RoleTemplateIdentifiers {
            api_group: api_group.to_string(),
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

fn generate_cluster_role_file(resources: Vec<String>, api_group: &str) {
    if get_ignored_files().contains(&format!("{}/clusterrole.yaml", RBAC_DIR)) {
        return;
    }

    let content = ClusterRoleTemplate {
        identifiers: ClusterRoleTemplateIdentifiers {
            api_group: api_group.to_string(),
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
    resources: Vec<Resource>,
}

#[derive(Serialize, Deserialize)]
struct Resource {
    api_group: String,
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
    examples: std::collections::HashMap<String, ReferenceOr<openapiv3::Example>>,
    operator_group: &str,
    operator_version: &str,
    operator_resource_ref: &str,
) {
    let examples_map: std::collections::HashMap<String, openapiv3::Example> = examples
        .into_iter()
        .filter_map(|(k, v)| match v {
            ReferenceOr::Item(item) => Some((k, item)),
            ReferenceOr::Reference { .. } => None,
        })
        .collect();
    for (name, example) in &examples_map {
        generate_manifest_from_example(
            &name,
            example,
            operator_group,
            operator_version,
            operator_resource_ref,
        );
    }
}

fn generate_manifest_from_example(
    name: &str,
    example: &openapiv3::Example,
    operator_group: &str,
    operator_version: &str,
    operator_resource_ref: &str,
) {
    let mut resources = Vec::new();

    match &example.value {
        Some(Value::Object(map)) => {
            let metadata_name = get_metadata_name(&name, map);
            resources.push(generate_resource_from_map(
                &name,
                &metadata_name,
                map,
                operator_group,
                operator_version,
                operator_resource_ref,
            ));
        }
        Some(Value::Array(arr)) => {
            for value in arr {
                if let Value::Object(map) = value {
                    let metadata_name = get_metadata_name(&name, map);
                    resources.push(generate_resource_from_map(
                        &name,
                        &metadata_name,
                        map,
                        operator_group,
                        operator_version,
                        operator_resource_ref,
                    ));
                }
            }
        }
        _ => (),
    }

    if !resources.is_empty() {
        write_manifest(name, resources);
    }
}

fn get_metadata_name(name: &str, map: &Map<String, Value>) -> String {
    map.get("x-kubernetes-operator-example-metadata")
        .cloned()
        .and_then(|v| serde_json::from_value::<Metadata>(v).ok())
        .and_then(|meta| Some(meta.name.clone()))
        .unwrap_or_else(|| {
            warn!("Warning: x-kubernetes-operator-example-metadata is not set for example {}. Using the regular example name.", name);
            name.to_lowercase()
        })
}

fn generate_resource_from_map(
    kind: &str,
    name: &str,
    map: &serde_json::Map<String, Value>,
    operator_group: &str,
    operator_version: &str,
    operator_resource_ref: &str,
) -> Resource {
    let mut map = map.clone();

    // Filter out resource remote reference from the examples
    map.remove(operator_resource_ref);

    // Filter out fields starting with 'x-'
    map.retain(|key, _| !key.starts_with("x-"));

    let json_value = Value::Object(map);
    let yaml_value = json!(json_value);
    let mut yaml_string = serde_yaml::to_string(&yaml_value).unwrap_or_else(|_| String::from("{}"));
    yaml_string = yaml_string
        .lines()
        .map(|line| format!("  {}", line))
        .collect::<Vec<_>>()
        .join("\n");

    Resource {
        api_group: operator_group.to_string(),
        api_version: operator_version.to_string(),
        kind: uppercase_first_letter(kind).to_singular(),
        metadata: Metadata {
            name: format!("example-{}", name.to_lowercase()),
        },
        spec: yaml_string,
    }
}

fn write_manifest(name: &str, resources: Vec<Resource>) {
    let template = ExampleTemplate { resources };

    match template.render() {
        Ok(content) => {
            write_to_file(
                format!("{}/{}.yaml", EXAMPLES_DIR, name.to_lowercase()),
                content,
            );
        }
        Err(e) => {
            error!("Failed to render template: {}", e);
        }
    }
}
