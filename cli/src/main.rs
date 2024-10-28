use askama::Template;
use indexmap::IndexMap;
use inflector::Inflector;
use log::{error, info, warn};
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use serde_yaml::Value as YamlValue;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::{DirBuilder, File, OpenOptions},
    io::{BufRead, BufReader, Error, Write},
    path::Path,
    process::Command as ProcessCommand,
    vec,
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "Kubernetes Operator Codegen",
    version = "1.0",
    author = "Eden Reich <eden.reich@gmail.com>",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize the directory structure")]
    Init {},
    #[command(about = "Hydrate the OpenAPI specification with x-kubernetes-operator-* extensions")]
    Hydrate {
        #[arg(required = true, help = "Path to the OpenAPI specification file")]
        openapi_file: String,
    },
    #[command(about = "Generate Kubernetes operator code from an OpenAPI specification")]
    Generate {
        #[arg(required = true, help = "Path to the OpenAPI specification file")]
        openapi_file: String,
        #[arg(short, long, help = "Generate all code")]
        all: bool,
        #[arg(short, long, help = "Generate the lib.rs file")]
        lib: bool,
        #[arg(short, long, help = "Generate the manifests")]
        manifests: bool,
        #[arg(short, long, help = "Generate the controllers")]
        controllers: bool,
        #[arg(short, long, help = "Generate the types")]
        types: bool,
    },
}

const CARGO_DIR: &str = ".cargo";
const DEVCONTAINER_DIR: &str = ".devcontainer";
const K8S_OPERATOR_DIR: &str = "operator";
const K8S_CRDGEN_DIR: &str = "crdgen";
const K8S_TESTS_DIR: &str = "tests";
const K8S_TESTS_UTILS_DIR: &str = "tests/src/utils";
const K8S_OPERATOR_CONTROLLERS_DIR: &str = "operator/src/controllers";
const K8S_OPERATOR_TYPES_DIR: &str = "operator/src/types";
const K8S_MANIFESTS_RBAC_DIR: &str = "manifests/rbac";
const K8S_MANIFESTS_EXAMPLES_DIR: &str = "manifests/examples";

fn main() {
    dotenv::dotenv().ok();

    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init {}) => {
            info!("Initializing directory structure...");
            generate_openapi_generator_ignore();

            create_directory_if_not_exists(CARGO_DIR);
            create_directory_if_not_exists(DEVCONTAINER_DIR);

            create_directory_if_not_exists(K8S_OPERATOR_DIR);
            create_directory_if_not_exists(K8S_CRDGEN_DIR);
            create_directory_if_not_exists(K8S_TESTS_DIR);

            generate_cargo_toml();
            generate_k8s_operator_cargo_toml();
            generate_k8s_crdgen_cargo_toml();
            generate_k8s_tests_cargo_toml();

            generate_dockerignore();
            generate_editorconfig();
            generate_envexample();
            generate_gitattributes();
            generate_gitignore();
            generate_prettierrc();
            generate_rustfmt_toml();
            generate_cluster_yaml();
            generate_dockerfile();
            generate_readme_md();
            generate_taskfile();
            generate_devcontainer_json();
            generate_devcontainer_deps();
            generate_devcontainer_launch_json();
            generate_devcontainer_zshrc();
            generate_cargo_config();

            create_directory_if_not_exists(K8S_OPERATOR_CONTROLLERS_DIR);
            create_directory_if_not_exists(K8S_OPERATOR_TYPES_DIR);
            create_directory_if_not_exists(K8S_TESTS_UTILS_DIR);

            create_file_if_not_exists(K8S_OPERATOR_CONTROLLERS_DIR, "mod.rs");
            create_file_if_not_exists(K8S_OPERATOR_TYPES_DIR, "mod.rs");
            create_file_if_not_exists(K8S_TESTS_UTILS_DIR, "mod.rs");

            generate_tests_utils_client();
            generate_tests_utils_operator();
            generate_tests_utils_cluster();

            add_tests_util_to_modfile("client").expect("Failed to add client util to mod file");
            add_tests_util_to_modfile("operator").expect("Failed to add operator util to mod file");
            add_tests_util_to_modfile("cluster").expect("Failed to add cluster util to mod file");

            // TODO - make the tests generated dynamically based on the defined API controllers and types.
            generate_tests();

            create_directory_if_not_exists(K8S_MANIFESTS_RBAC_DIR);
            create_directory_if_not_exists(K8S_MANIFESTS_EXAMPLES_DIR);
        }
        Some(Commands::Hydrate { openapi_file }) => {
            info!("Hydrating OpenAPI spec...");
            let mut openapi = read_openapi_spec_raw(openapi_file);

            if let Some(info) = openapi.get_mut("info") {
                if let Some(info_map) = info.as_mapping_mut() {
                    info_map.insert(
                        YamlValue::String("x-kubernetes-operator-group".to_string()),
                        YamlValue::String(
                            std::env::var("KUBERNETES_OPERATOR_GROUP").unwrap_or_default(),
                        ),
                    );
                    info_map.insert(
                        YamlValue::String("x-kubernetes-operator-version".to_string()),
                        YamlValue::String(
                            std::env::var("KUBERNETES_OPERATOR_VERSION").unwrap_or_default(),
                        ),
                    );
                    info_map.insert(
                        YamlValue::String("x-kubernetes-operator-resource-ref".to_string()),
                        YamlValue::String(
                            std::env::var("KUBERNETES_OPERATOR_RESOURCE_REF").unwrap_or_default(),
                        ),
                    );
                    let kubernetes_operator_include_tags = std::env::var(
                        "KUBERNETES_OPERATOR_INCLUDE_TAGS",
                    )
                    .expect("KUBERNETES_OPERATOR_INCLUDE_TAGS environment variable not set");
                    let tags_list: Vec<YamlValue> = kubernetes_operator_include_tags
                        .split(',')
                        .map(|tag| YamlValue::String(tag.trim().to_string()))
                        .collect();
                    info_map.insert(
                        YamlValue::String("x-kubernetes-operator-include-tags".to_string()),
                        YamlValue::Sequence(tags_list),
                    );
                    info_map.insert(
                        YamlValue::String(
                            "x-kubernetes-operator-example-metadata-spec-field-ref".to_string(),
                        ),
                        YamlValue::String(
                            std::env::var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF")
                                .unwrap_or_default(),
                        ),
                    );
                }
            }

            write_openapi_spec_raw(openapi_file, &openapi);

            let _ = ProcessCommand::new("prettier")
                .arg("--write")
                .arg(openapi_file)
                .output()
                .expect("Failed to run prettier on OpenAPI spec");

            info!("OpenAPI spec hydrated successfully");
        }
        Some(Commands::Generate {
            openapi_file,
            all,
            lib,
            manifests,
            controllers,
            types,
        }) => {
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
                        openapiv3::ReferenceOr::Item(schema) => {
                            Some((name.clone(), schema.clone()))
                        }
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
                    kubernetes_operator_include_tags,
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
                return;
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
                    kubernetes_operator_include_tags,
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
        }
        None => {
            info!("No command provided");
        }
    }
}

fn read_openapi_spec_raw(file_path: &str) -> IndexMap<String, YamlValue> {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Unable to parse OpenAPI spec")
}

fn read_openapi_spec(file_path: &str) -> OpenAPI {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Unable to parse OpenAPI spec")
}

fn write_openapi_spec_raw(file_path: &str, openapi: &IndexMap<String, YamlValue>) {
    let file = File::create(file_path).expect("Unable to create file");
    serde_yaml::to_writer(file, openapi).expect("Unable to write OpenAPI spec");
}

fn extract_openapi_info(openapi: &OpenAPI) -> (String, String, String, Vec<String>, String) {
    let kubernetes_operator_group = extract_extension(openapi, "x-kubernetes-operator-group");
    let kubernetes_operator_version = extract_extension(openapi, "x-kubernetes-operator-version");
    let kubernetes_operator_resource_ref =
        extract_extension(openapi, "x-kubernetes-operator-resource-ref");
    let kubernetes_operator_include_tags =
        extract_extension_array(openapi, "x-kubernetes-operator-include-tags");
    let kubernetes_operator_metadata_spec_field_name = extract_extension(
        openapi,
        "x-kubernetes-operator-example-metadata-spec-field-ref",
    );
    (
        kubernetes_operator_group,
        kubernetes_operator_version,
        kubernetes_operator_resource_ref,
        kubernetes_operator_include_tags,
        kubernetes_operator_metadata_spec_field_name,
    )
}

fn extract_extension(openapi: &OpenAPI, key: &str) -> String {
    openapi
        .info
        .extensions
        .get(key)
        .unwrap_or_else(|| panic!("No {} in OpenAPI spec", key))
        .as_str()
        .unwrap_or_else(|| panic!("{} is not a string", key))
        .to_string()
}

fn extract_extension_array(openapi: &OpenAPI, key: &str) -> Vec<String> {
    openapi
        .info
        .extensions
        .get(key)
        .unwrap_or_else(|| panic!("No {} in OpenAPI spec", key))
        .as_array()
        .unwrap_or_else(|| panic!("{} is not an array", key))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

fn create_directory_if_not_exists(dir: &str) {
    let path = Path::new(dir);
    if path.exists() {
        info!("Directory {} already exists. skipping...", dir);
    } else {
        DirBuilder::new()
            .recursive(true)
            .create(dir)
            .unwrap_or_else(|_| panic!("Unable to create {} directory", dir));
    }
}

fn create_file_if_not_exists(dir: &str, file: &str) {
    let file_path = format!("{}/{}", dir, file);
    if !std::path::Path::new(&file_path).exists() {
        File::create(&file_path).unwrap_or_else(|_| panic!("Unable to create file {}", file_path));
    }
}

#[derive(Template)]
#[template(path = "k8s_operator_cargo.toml.jinja")]
struct K8sOperatorCargoTomlTemplate {}

fn generate_k8s_operator_cargo_toml() {
    let content = K8sOperatorCargoTomlTemplate {}.render().unwrap();
    write_to_file(format!("{}/Cargo.toml", K8S_OPERATOR_DIR), content);
}

#[derive(Template)]
#[template(path = "k8s_crdgen_cargo.toml.jinja")]
struct K8sCrdgenCargoTomlTemplate {}

fn generate_k8s_crdgen_cargo_toml() {
    let content = K8sCrdgenCargoTomlTemplate {}.render().unwrap();
    write_to_file(format!("{}/Cargo.toml", K8S_CRDGEN_DIR), content);
}

#[derive(Template)]
#[template(path = "k8s_tests_cargo.toml.jinja")]
struct K8sTestsCargoTomlTemplate {}

fn generate_k8s_tests_cargo_toml() {
    let content = K8sTestsCargoTomlTemplate {}.render().unwrap();
    write_to_file(format!("{}/Cargo.toml", K8S_TESTS_DIR), content);
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
    if get_ignored_files().contains(&format!("{}/src/main.rs", K8S_OPERATOR_DIR)) {
        return;
    }

    controllers.sort();

    let content: String = MainTemplate {
        api_group: api_group.into(),
        api_version: api_version.into(),
        controllers,
    }
    .render()
    .unwrap();
    write_to_file(format!("{}/src/main.rs", K8S_OPERATOR_DIR), content);
    format_file(format!("{}/src/main.rs", K8S_OPERATOR_DIR));
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

struct Field {
    pub_name: String,
    field_type: String,
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

    let file_path: String = format!("{}/{}.rs", K8S_OPERATOR_CONTROLLERS_DIR, tag.to_lowercase());
    write_to_file(file_path.to_string(), content);
    format_file(file_path.to_string());
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

    let file_path = format!("{}/{}.rs", K8S_OPERATOR_TYPES_DIR, arg_name_clone);
    write_to_file(file_path.to_string(), content);
    format_file(file_path.to_string());
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

    write_to_file(file_path.clone(), content);
    format_file(file_path);
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

fn add_tests_util_to_modfile(util_name: &str) -> Result<(), Error> {
    let file_path = format!("{}/mod.rs", K8S_TESTS_UTILS_DIR);
    match upsert_line_to_file(
        file_path.as_str(),
        format!("pub mod {};", util_name.to_lowercase()).as_str(),
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
    let filepath = format!("{}/src/main.rs", K8S_CRDGEN_DIR);
    if get_ignored_files().contains(&filepath) {
        return;
    }

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
    write_to_file(filepath.clone(), content);
    format_file(filepath);
}

#[derive(Template)]
#[template(path = "k8s_tests_main.jinja")]
struct TestsMainTemplate {}

fn generate_tests() {
    let content = TestsMainTemplate {}.render().unwrap();
    write_to_file(format!("{}/src/main.rs", K8S_TESTS_DIR), content);
}

#[derive(Template)]
#[template(path = "k8s_tests_utils_client.jinja")]
struct TestsUtilsClientTemplate {}

fn generate_tests_utils_client() {
    let content = TestsUtilsClientTemplate {}.render().unwrap();
    write_to_file(format!("{}/client.rs", K8S_TESTS_UTILS_DIR), content);
}

#[derive(Template)]
#[template(path = "k8s_tests_utils_operator.jinja")]
struct TestsUtilsOperatorTemplate {}

fn generate_tests_utils_operator() {
    let content = TestsUtilsOperatorTemplate {}.render().unwrap();
    write_to_file(format!("{}/operator.rs", K8S_TESTS_UTILS_DIR), content);
}

#[derive(Template)]
#[template(path = "k8s_tests_utils_cluster.jinja")]
struct TestsUtilsClusterTemplate {}

fn generate_tests_utils_cluster() {
    let content = TestsUtilsClusterTemplate {}.render().unwrap();
    write_to_file(format!("{}/cluster.rs", K8S_TESTS_UTILS_DIR), content);
}

#[derive(Template)]
#[template(path = ".dockerignore.jinja")]
struct DockerignoreTemplate {}

fn generate_dockerignore() {
    let content = DockerignoreTemplate {}.render().unwrap();
    write_to_file(String::from(".dockerignore"), content);
}

#[derive(Template)]
#[template(path = ".editorconfig.jinja")]
struct EditorconfigTemplate {}

fn generate_editorconfig() {
    let content = EditorconfigTemplate {}.render().unwrap();
    write_to_file(String::from(".editorconfig"), content);
}

#[derive(Template)]
#[template(path = ".env.example.jinja")]
struct EnvExampleTemplate {}

fn generate_envexample() {
    let content = EnvExampleTemplate {}.render().unwrap();
    write_to_file(String::from(".env.example"), content);
}

#[derive(Template)]
#[template(path = ".gitattributes.jinja")]
struct GitattributesTemplate {}

fn generate_gitattributes() {
    let content = GitattributesTemplate {}.render().unwrap();
    write_to_file(String::from(".gitattributes"), content);
}

#[derive(Template)]
#[template(path = ".gitignore.jinja")]
struct GitignoreTemplate {}

fn generate_gitignore() {
    let content = GitignoreTemplate {}.render().unwrap();
    write_to_file(String::from(".gitignore"), content);
}

#[derive(Template)]
#[template(path = ".openapi-generator-ignore.jinja")]
struct OpenAPIGeneratorIgnoreTemplate {}

fn generate_openapi_generator_ignore() {
    let content = OpenAPIGeneratorIgnoreTemplate {}.render().unwrap();
    write_to_file_without_filter(String::from(".openapi-generator-ignore"), content);
}

#[derive(Template)]
#[template(path = ".prettierrc.jinja")]
struct PrettierrcTemplate {}

fn generate_prettierrc() {
    let content = PrettierrcTemplate {}.render().unwrap();
    write_to_file(String::from(".prettierrc"), content);
}

#[derive(Template)]
#[template(path = ".rustfmt.toml.jinja")]
struct RustfmtTomlTemplate {}

fn generate_rustfmt_toml() {
    let content = RustfmtTomlTemplate {}.render().unwrap();
    write_to_file(String::from(".rustfmt.toml"), content);
}

#[derive(Template)]
#[template(path = "cargo.toml.jinja")]
struct CargoTomlTemplate {}

fn generate_cargo_toml() {
    let content = CargoTomlTemplate {}.render().unwrap();
    write_to_file(String::from("Cargo.toml"), content);
}

#[derive(Template)]
#[template(path = "cluster.yaml.jinja")]
struct ClusterYamlTemplate {}

fn generate_cluster_yaml() {
    let content = ClusterYamlTemplate {}.render().unwrap();
    write_to_file(String::from("Cluster.yaml"), content);
}

#[derive(Template)]
#[template(path = "dockerfile.jinja")]
struct DockerfileTemplate {}

fn generate_dockerfile() {
    let content = DockerfileTemplate {}.render().unwrap();
    write_to_file(String::from("Dockerfile"), content);
}

#[derive(Template)]
#[template(path = "readme.md.jinja")]
struct ReadmeMdTemplate {}

fn generate_readme_md() {
    let content = ReadmeMdTemplate {}.render().unwrap();
    write_to_file(String::from("README.md"), content);
}

#[derive(Template)]
#[template(path = "taskfile.jinja")]
struct TaskfileTemplate {}

fn generate_taskfile() {
    let content = TaskfileTemplate {}.render().unwrap();
    write_to_file(String::from("Taskfile"), content);
}

#[derive(Template)]
#[template(path = ".devcontainer_devcontainer.json.jinja")]
struct DevcontainerJsonTemplate {}

fn generate_devcontainer_json() {
    let content = DevcontainerJsonTemplate {}.render().unwrap();
    write_to_file(String::from(".devcontainer/devcontainer.json"), content);
}

#[derive(Template)]
#[template(path = ".devcontainer_deps.sh.jinja")]
struct DevcontainerDepsTemplate {}

fn generate_devcontainer_deps() {
    let content = DevcontainerDepsTemplate {}.render().unwrap();
    write_to_file(String::from(".devcontainer/deps.sh"), content);
}

#[derive(Template)]
#[template(path = ".devcontainer_launch.json.jinja")]
struct DevcontainerLaunchJsonExampleTemplate {}

fn generate_devcontainer_launch_json() {
    let content = DevcontainerLaunchJsonExampleTemplate {}.render().unwrap();
    write_to_file(String::from(".devcontainer/launch.json"), content);
}

#[derive(Template)]
#[template(path = ".devcontainer_zshrc.jinja")]
struct DevcontainerZshrcTemplate {}

fn generate_devcontainer_zshrc() {
    let content = DevcontainerZshrcTemplate {}.render().unwrap();
    write_to_file(String::from(".devcontainer/.zshrc"), content);
}

#[derive(Template)]
#[template(path = ".cargo_config.toml.jinja")]
struct CargoConfigTemplate {}

fn generate_cargo_config() {
    let content = CargoConfigTemplate {}.render().unwrap();
    write_to_file(String::from(".cargo/config.toml"), content);
}

fn get_ignored_files() -> Vec<String> {
    let ignore_file =
        std::fs::File::open(".openapi-generator-ignore").expect("Unable to open file");
    let reader = BufReader::new(ignore_file);
    reader.lines().map_while(Result::ok).collect()
}

fn write_to_file(file_path: String, file_content: String) {
    if get_ignored_files().contains(&file_path) {
        return;
    }

    std::fs::write(file_path, file_content + "\n").expect("Unable to write file");
}

fn write_to_file_without_filter(file_path: String, file_content: String) {
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
    if get_ignored_files().contains(&format!("{}/role.yaml", K8S_MANIFESTS_RBAC_DIR)) {
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
    write_to_file(format!("{}/role.yaml", K8S_MANIFESTS_RBAC_DIR), content);
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
    if get_ignored_files().contains(&format!("{}/clusterrole.yaml", K8S_MANIFESTS_RBAC_DIR)) {
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
    write_to_file(
        format!("{}/clusterrole.yaml", K8S_MANIFESTS_RBAC_DIR),
        content,
    );
}

#[derive(Template)]
#[template(path = "manifest_rbac_service_account.jinja")]
struct ServiceAccountTemplate {}

fn generate_service_account_file() {
    if get_ignored_files().contains(&format!("{}/serviceaccount.yaml", K8S_MANIFESTS_RBAC_DIR)) {
        return;
    }

    let content = ServiceAccountTemplate {}.render().unwrap();
    write_to_file(
        format!("{}/serviceaccount.yaml", K8S_MANIFESTS_RBAC_DIR),
        content,
    );
}

#[derive(Template)]
#[template(path = "manifest_rbac_role_binding.jinja")]
struct RoleBindingTemplate {}

fn generate_role_binding_file_content() {
    if get_ignored_files().contains(&format!("{}/rolebinding.yaml", K8S_MANIFESTS_RBAC_DIR)) {
        return;
    }

    let content = RoleBindingTemplate {}.render().unwrap();
    write_to_file(
        format!("{}/rolebinding.yaml", K8S_MANIFESTS_RBAC_DIR),
        content,
    );
}

#[derive(Template)]
#[template(path = "manifest_rbac_cluster_role_binding.jinja")]
struct ClusterRoleBindingTemplate {}

fn generate_cluster_role_binding_file_content() {
    if get_ignored_files().contains(&format!(
        "{}/clusterrolebinding.yaml",
        K8S_MANIFESTS_RBAC_DIR
    )) {
        return;
    }

    let content = ClusterRoleBindingTemplate {}.render().unwrap();
    write_to_file(
        format!("{}/clusterrolebinding.yaml", K8S_MANIFESTS_RBAC_DIR),
        content,
    );
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

#[derive(Template)]
#[template(path = "manifest_operator_secret.jinja")]
struct OperatorSecretTemplate {}

fn generate_operator_secret_file() {
    if get_ignored_files().contains(&"manifests/operator/secret.yaml".to_string()) {
        return;
    }

    let content = OperatorSecretTemplate {}.render().unwrap();
    write_to_file("manifests/operator/secret.yaml".to_string(), content);
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
        write_manifest(name, resources);
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

fn write_manifest(name: &str, resources: Vec<Resource>) {
    let template = ExampleTemplate { resources };

    match template.render() {
        Ok(content) => {
            write_to_file(
                format!(
                    "{}/{}.yaml",
                    K8S_MANIFESTS_EXAMPLES_DIR,
                    name.to_lowercase()
                ),
                content,
            );
        }
        Err(e) => {
            error!("Failed to render template: {}", e);
        }
    }
}
