use crate::utils::{extract_openapi_info, read_openapi_spec};
use askama::Template;
use clap::Error;
use inflector::Inflector;
use log::{debug, error, info, warn};
use openapiv3::{ReferenceOr, Schema, SchemaKind, Type};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::process::Command as ProcessCommand;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
};

const K8S_OPERATOR_DIR: &str = "operator";
const K8S_CRDGEN_DIR: &str = "crdgen";

const K8S_OPERATOR_TYPES_DIR: &str = "operator/src/types";
const K8S_OPERATOR_CONTROLLERS_DIR: &str = "operator/src/controllers";

const K8S_MANIFESTS_RBAC_DIR: &str = "manifests/rbac";
const K8S_MANIFESTS_OPERATOR_DIR: &str = "manifests/operator";
const K8S_MANIFESTS_EXAMPLES_DIR: &str = "manifests/examples";

pub fn execute(
    openapi_file: &String,
    all: &bool,
    lib: &bool,
    manifests: &bool,
    controllers: &bool,
    types: &bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Using OpenAPI file: {}", openapi_file);

    let openapi = read_openapi_spec(openapi_file);
    let (
        kubernetes_operator_group,
        kubernetes_operator_version,
        kubernetes_operator_resource_ref,
        kubernetes_operator_include_tags,
        kubernetes_operator_metadata_spec_field_name,
    ) = extract_openapi_info(&openapi);
    let components = openapi
        .components
        .clone()
        .expect("No components in OpenAPI spec");
    let paths: openapiv3::Paths = openapi.paths.clone();

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

    if *all || (!*lib && !*manifests && !*controllers && !*types) {
        info!("Generating all...");
        generate_lib();
        generate_types(schemas.clone(), &kubernetes_operator_resource_ref);
        let controllers = generate_controllers(
            schemas.clone(),
            paths.clone(),
            kubernetes_operator_include_tags.clone(),
            kubernetes_operator_resource_ref.clone(),
        );
        generate_main_file(
            &kubernetes_operator_group,
            &kubernetes_operator_version,
            controllers,
        );
        generate_rbac_files(schema_names.clone(), &kubernetes_operator_group);
        generate_crdgen_file(schema_names.clone());
        generate_examples(
            &kubernetes_operator_metadata_spec_field_name,
            components.examples.into_iter().collect(),
            &kubernetes_operator_group,
            &kubernetes_operator_version,
            &kubernetes_operator_resource_ref.clone(),
        );
        return Ok(());
    }
    if *lib {
        info!("Generating lib...");
        generate_lib();
    }
    if *manifests {
        info!("Generating manifests...");
        generate_rbac_files(schema_names.clone(), &kubernetes_operator_group);
        generate_crdgen_file(schema_names.clone());
        generate_examples(
            &kubernetes_operator_metadata_spec_field_name,
            components.examples.into_iter().collect(),
            &kubernetes_operator_group,
            &kubernetes_operator_version,
            &kubernetes_operator_resource_ref.clone(),
        );
    }
    if *controllers {
        info!("Generating controllers...");
        let controllers = generate_controllers(
            schemas.clone(),
            paths.clone(),
            kubernetes_operator_include_tags.clone(),
            kubernetes_operator_resource_ref.clone(),
        );
        generate_main_file(
            &kubernetes_operator_group,
            &kubernetes_operator_version,
            controllers,
        );
    }
    if *types {
        info!("Generating the types...");
        generate_types(schemas.clone(), &kubernetes_operator_resource_ref);
    }
    Ok(())
}

fn generate_rbac_files(resources: Vec<String>, kubernetes_operator_group: &str) {
    generate_role_file(resources.clone(), kubernetes_operator_group);
    generate_cluster_role_file(resources.clone(), kubernetes_operator_group);
    generate_service_account_file();
    generate_role_binding_file_content();
    generate_cluster_role_binding_file_content();
    generate_operator_deployment_file();
    generate_operator_secret_file();
}

#[derive(Template)]
#[template(path = "k8s_operator_main.jinja")]
struct MainTemplate {
    api_group: String,
    api_version: String,
    controllers: Vec<String>,
}

fn generate_main_file(api_group: &str, api_version: &str, mut controllers: Vec<String>) {
    let base_path = &Path::new(K8S_OPERATOR_DIR).join("src");
    let file_path = base_path.join("main.rs").to_string_lossy().to_string();
    if get_ignored_files().contains(&file_path) {
        return;
    }

    controllers.sort();

    let base_path = &Path::new(K8S_OPERATOR_DIR).join("src");
    let content: String = MainTemplate {
        api_group: api_group.into(),
        api_version: api_version.into(),
        controllers,
    }
    .render()
    .unwrap();
    write_to_file(base_path, "main.rs", content);
    format_file(base_path.join("main.rs").to_string_lossy().to_string());
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
) -> Vec<String> {
    let mut controllers: HashMap<String, Vec<ControllerAttributes>> = HashMap::new();

    for (_path, path_item) in paths {
        let item = if let ReferenceOr::Item(item) = path_item {
            item
        } else {
            continue;
        };

        let operations = vec![
            ("get", &item.get),
            ("post", &item.post),
            ("delete", &item.delete),
            ("put", &item.put),
        ];

        for (method, operation) in operations {
            let operation = match operation {
                Some(operation) => operation,
                None => continue,
            };

            if operation
                .tags
                .iter()
                .all(|tag: &String| !include_tags.contains(tag))
            {
                continue;
            }

            if let Some((tag, controller)) =
                get_controller_attributes_for_operation(operation, method, &include_tags)
            {
                controllers.entry(tag.clone()).or_default().push(controller);
            }
        }
    }

    for (tag, controller_attributes) in &controllers {
        generate_controller(
            schemas.clone(),
            tag.clone(),
            controller_attributes,
            kubernetes_operator_resource_ref.clone(),
        );

        if let Err(e) = upsert_line_to_file(
            ".openapi-generator-ignore",
            format!("{}/{}.rs", K8S_OPERATOR_CONTROLLERS_DIR, tag.to_lowercase()).as_str(),
        ) {
            error!(
                "Failed to add controller to .openapi-generator-ignore file: {:?}",
                e
            );
        }
    }

    controllers.keys().cloned().collect()
}

struct Field {
    pub_name: String,
    field_type: String,
}

#[derive(Template)]
#[template(path = "k8s_operator_controller.jinja")]
struct ControllerTemplate {
    tag: String,
    arg_name: String,
    kind_struct: String,
    dto_fields: Vec<Field>,
    resource_remote_ref: String,
    has_create_action: bool,
    has_update_action: bool,
    has_delete_action: bool,
    api_url: String,
}

#[derive(Template)]
#[template(path = "k8s_operator_controller_action_delete.jinja")]
struct ControllerActionDeleteTemplate<'a> {
    arg_name: String,
    kind_struct: String,
    controllers: Vec<&'a ControllerAttributes>,
    resource_remote_ref: String,
}

#[derive(Template)]
#[template(path = "k8s_operator_controller_action_update.jinja")]
struct ControllerActionPutTemplate<'a> {
    arg_name: String,
    kind_struct: String,
    controllers: Vec<&'a ControllerAttributes>,
    resource_remote_ref: String,
}

#[derive(Template)]
#[template(path = "k8s_operator_controller_action_create.jinja")]
struct ControllerActionPostTemplate<'a> {
    arg_name: String,
    kind_struct: String,
    controllers: Vec<&'a ControllerAttributes>,
    resource_remote_ref: String,
}

fn generate_controller(
    schemas: HashMap<String, Schema>,
    tag: String,
    controller_attributes: &[ControllerAttributes],
    resource_remote_ref: String,
) {
    if get_ignored_files().contains(&format!(
        "{}/{}.rs",
        K8S_OPERATOR_CONTROLLERS_DIR,
        tag.to_lowercase()
    )) {
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
        dto_fields: fields,
        resource_remote_ref: resource_remote_ref.clone(),
        has_create_action,
        has_update_action,
        has_delete_action,
        api_url: "http://localhost:8080".to_string(),
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

    let base_path: &Path = Path::new(K8S_OPERATOR_CONTROLLERS_DIR);
    let file_name: String = format!("{}.rs", tag.to_lowercase());
    write_to_file(base_path, &file_name, content);
    format_file(base_path.join(file_name).to_string_lossy().to_string());
    add_controller_to_modfile(&tag.to_lowercase()).expect("Failed to add controller to mod file");
}

#[derive(Template)]
#[template(path = "k8s_operator_type.jinja")]
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
            name,
            "example.com",
            "v1",
            operator_resource_ref,
        );
        match add_type_to_modfile(name) {
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
    if get_ignored_files().contains(&format!(
        "{}/{}.rs",
        K8S_OPERATOR_TYPES_DIR,
        name.to_lowercase()
    )) {
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

    let base_path: &Path = Path::new(K8S_OPERATOR_TYPES_DIR);
    let file_name: String = format!("{}.rs", arg_name_clone);
    write_to_file(base_path, &file_name, content);
    format_file(base_path.join(file_name).to_string_lossy().to_string());
}
#[derive(Template)]
#[template(path = "k8s_operator_lib.jinja")]
struct LibTemplate {}

fn generate_lib() {
    let file_path = format!("{}/src/lib.rs", K8S_OPERATOR_DIR);
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let content: String = LibTemplate {}.render().unwrap();

    let base_path: &Path = &Path::new(K8S_OPERATOR_DIR).join("src");
    let file_name: String = "lib.rs".to_string();
    write_to_file(base_path, &file_name, content);
    format_file(base_path.join(file_name).to_string_lossy().to_string());
}

fn add_type_to_modfile(type_name: &str) -> Result<(), Error> {
    let file_path = format!("{}/mod.rs", K8S_OPERATOR_TYPES_DIR);
    match upsert_line_to_file(
        file_path.as_str(),
        format!("pub mod {};", type_name.to_lowercase()).as_str(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

fn add_controller_to_modfile(controller_name: &str) -> Result<(), Error> {
    let file_path = format!("{}/mod.rs", K8S_OPERATOR_CONTROLLERS_DIR);
    match upsert_line_to_file(
        file_path.as_str(),
        format!("pub mod {};", controller_name.to_lowercase()).as_str(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

#[derive(Template)]
#[template(path = "k8s_crdgen_main.jinja")]
struct CrdGenTemplate {
    resources: BTreeMap<String, String>,
}

fn generate_crdgen_file(resources: Vec<String>) {
    let base_path: &Path = &Path::new(K8S_CRDGEN_DIR).join("src");
    let file_name = "main.rs".to_string();

    let resources: BTreeMap<String, String> = resources
        .into_iter()
        .map(|resource| {
            (
                resource.clone().to_singular(),
                resource.to_singular().to_class_case(),
            )
        })
        .collect();

    let template = CrdGenTemplate { resources };
    let content = template.render().unwrap();
    write_to_file(base_path, &file_name, content);
    format_file(base_path.join(file_name).to_string_lossy().to_string());
}

fn get_ignored_files() -> Vec<String> {
    let ignore_file_path = ".openapi-generator-ignore";
    let ignore_file = File::open(ignore_file_path)
        .unwrap_or_else(|_| panic!("Unable to open file: {:?}", ignore_file_path));
    let reader = BufReader::new(ignore_file);
    reader.lines().map_while(Result::ok).collect()
}

fn write_to_file(base_path: &Path, file_name: &str, file_content: String) {
    let file_path = base_path.join(file_name);
    debug!("Writing to file: {}", file_path.to_string_lossy());
    if get_ignored_files().contains(&file_path.to_string_lossy().to_string()) {
        return;
    }

    std::fs::write(file_path, file_content + "\n").expect("Unable to write file");
}

fn upsert_line_to_file(file_path: &str, line: &str) -> Result<(), Error> {
    if get_ignored_files().contains(&file_path.to_string()) {
        return Ok(());
    }

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let exists = reader.lines().any(|l| l.unwrap() == line);

    if !exists {
        let mut file = OpenOptions::new().append(true).open(file_path)?;
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

    let output = ProcessCommand::new("rustfmt")
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
    let base_path: &Path = Path::new(K8S_MANIFESTS_RBAC_DIR);
    let file_name = "role.yaml";
    let file_path: String = base_path.join(file_name).to_string_lossy().to_string();
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let content = RoleTemplate {
        identifiers: RoleTemplateIdentifiers {
            api_group: api_group.to_string(),
            resources,
        },
    }
    .render()
    .unwrap();
    write_to_file(base_path, file_name, content);
}

struct ClusterRoleTemplateIdentifiers {
    api_group: String,
    resources: Vec<String>,
}

#[derive(Template)]
#[template(path = "manifest_rbac_cluster_role.jinja")]
struct ClusterRoleTemplate {
    identifiers: ClusterRoleTemplateIdentifiers,
}

fn generate_cluster_role_file(resources: Vec<String>, api_group: &str) {
    let base_path: &Path = Path::new(K8S_MANIFESTS_RBAC_DIR);
    let file_name = "clusterrole.yaml";
    let file_path: String = base_path.join(file_name).to_string_lossy().to_string();
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let content = ClusterRoleTemplate {
        identifiers: ClusterRoleTemplateIdentifiers {
            api_group: api_group.to_string(),
            resources,
        },
    }
    .render()
    .unwrap();
    write_to_file(base_path, file_name, content);
}

#[derive(Template)]
#[template(path = "manifest_rbac_service_account.jinja")]
struct ServiceAccountTemplate {}

fn generate_service_account_file() {
    let base_path: &Path = Path::new(K8S_MANIFESTS_RBAC_DIR);
    let file_name = "serviceaccount.yaml";
    let file_path: String = base_path.join(file_name).to_string_lossy().to_string();
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let content = ServiceAccountTemplate {}.render().unwrap();
    write_to_file(base_path, file_name, content);
}

#[derive(Template)]
#[template(path = "manifest_rbac_role_binding.jinja")]
struct RoleBindingTemplate {}

fn generate_role_binding_file_content() {
    let base_path: &Path = Path::new(K8S_MANIFESTS_RBAC_DIR);
    let file_name = "rolebinding.yaml";
    let file_path: String = base_path.join(file_name).to_string_lossy().to_string();
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let content = RoleBindingTemplate {}.render().unwrap();
    write_to_file(base_path, file_name, content);
}

#[derive(Template)]
#[template(path = "manifest_rbac_cluster_role_binding.jinja")]
struct ClusterRoleBindingTemplate {}

fn generate_cluster_role_binding_file_content() {
    let base_path: &Path = Path::new(K8S_MANIFESTS_RBAC_DIR);
    let file_name = "clusterrolebinding.yaml";
    let file_path: String = base_path.join(file_name).to_string_lossy().to_string();
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let content = ClusterRoleBindingTemplate {}.render().unwrap();
    write_to_file(base_path, file_name, content);
}

#[derive(Template)]
#[template(path = "manifest_operator_deployment.jinja")]
struct OperatorDeploymentTemplate {}

fn generate_operator_deployment_file() {
    let base_path: &Path = Path::new(K8S_MANIFESTS_OPERATOR_DIR);
    let file_name = "deployment.yaml";
    let file_path: String = base_path.join(file_name).to_string_lossy().to_string();
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let content = OperatorDeploymentTemplate {}.render().unwrap();
    write_to_file(base_path, file_name, content);
}

#[derive(Template)]
#[template(path = "manifest_operator_secret.jinja")]
struct OperatorSecretTemplate {}

fn generate_operator_secret_file() {
    let base_path: &Path = Path::new(K8S_MANIFESTS_OPERATOR_DIR);
    let file_name = "secret.yaml";
    let file_path: String = base_path.join(file_name).to_string_lossy().to_string();
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let content = OperatorSecretTemplate {}.render().unwrap();
    write_to_file(base_path, file_name, content);
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
    kubernetes_operator_metadata_spec_field_name: &str,
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
            kubernetes_operator_metadata_spec_field_name,
            name,
            example,
            operator_group,
            operator_version,
            operator_resource_ref,
        );
    }
}

fn generate_manifest_from_example(
    kubernetes_operator_metadata_spec_field_name: &str,
    name: &str,
    example: &openapiv3::Example,
    operator_group: &str,
    operator_version: &str,
    operator_resource_ref: &str,
) {
    let mut resources = Vec::new();

    match &example.value {
        Some(Value::Object(map)) => {
            let metadata_name =
                get_metadata_name(kubernetes_operator_metadata_spec_field_name, map);
            resources.push(generate_resource_from_map(
                name,
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
                    let metadata_name =
                        get_metadata_name(kubernetes_operator_metadata_spec_field_name, map);
                    resources.push(generate_resource_from_map(
                        name,
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
        write_example_manifest(name, resources);
    }
}

fn get_metadata_name(
    kubernetes_operator_metadata_spec_field_name: &str,
    map: &Map<String, Value>,
) -> String {
    let name = map.get(kubernetes_operator_metadata_spec_field_name);

    if let Some(Value::String(name)) = name {
        return name.to_lowercase();
    }

    warn!(
        "Warning: {} is not set for example. Using the regular example name.",
        kubernetes_operator_metadata_spec_field_name
    );

    kubernetes_operator_metadata_spec_field_name.to_lowercase()
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

fn write_example_manifest(name: &str, resources: Vec<Resource>) {
    let template = ExampleTemplate { resources };
    let base_path = Path::new(K8S_MANIFESTS_EXAMPLES_DIR);

    match template.render() {
        Ok(content) => {
            write_to_file(base_path, &format!("{}.yaml", name.to_lowercase()), content);
        }
        Err(e) => {
            error!("Failed to render template: {}", e);
        }
    }
}
