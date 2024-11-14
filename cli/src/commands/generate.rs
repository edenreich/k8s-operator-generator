use crate::errors::AppError;
use crate::templates::{
    ClusterRoleBindingTemplate, ClusterRoleTemplate, ClusterRoleTemplateIdentifiers,
    ControllerActionDeleteTemplate, ControllerActionPostTemplate, ControllerActionPutTemplate,
    ControllerAttributes, ControllerTemplate, CrdGenTemplate, ExampleTemplate, Field, LibTemplate,
    MainTemplate, Metadata, OperatorDeploymentTemplate, OperatorSecretTemplate, Resource,
    RoleBindingTemplate, RoleTemplate, RoleTemplateIdentifiers, ServiceAccountTemplate,
    TypeTemplate,
};
use crate::utils::{
    extract_openapi_info, format_file, generate_template_file, get_ignored_files,
    read_openapi_spec, uppercase_first_letter, upsert_line_to_file,
    validate_openapi_kubernetes_extensions_exists, write_to_file,
};
use askama::Template;
use inflector::Inflector;
use log::{error, info, warn};
use openapiv3::{ReferenceOr, Schema, SchemaKind, Type};
use serde_json::{json, Map, Value};
use std::fs::File;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::Path,
};

/// Executes the generation process based on the provided OpenAPI file and flags.
///
/// This function generates various components of a Kubernetes operator project
/// including library files, manifests, controllers, and types based on the
/// OpenAPI specification provided. The generation process can be controlled
/// using the provided flags to generate specific components or all components.
///
/// # Arguments
///
/// * `base_path` - A string slice that holds the base path for the project.
/// * `openapi_file` - A string slice that holds the path to the OpenAPI file.
/// * `all` - A boolean flag indicating whether to generate all components.
/// * `lib` - A boolean flag indicating whether to generate the library files.
/// * `manifests` - A boolean flag indicating whether to generate the manifest files.
/// * `controllers` - A boolean flag indicating whether to generate the controller files.
/// * `types` - A boolean flag indicating whether to generate the type files.
///
/// # Returns
///
/// This function returns a `Result` indicating the success or failure of the operation.
///
/// # Errors
///
/// This function will return an error if the OpenAPI file cannot be read or parsed,
/// or if any of the generation steps fail.
pub fn execute(
    base_path: &String,
    openapi_file: &String,
    all: &bool,
    lib: &bool,
    manifests: &bool,
    controllers: &bool,
    types: &bool,
) -> Result<(), AppError> {
    info!("Using OpenAPI file: {}", openapi_file);

    let openapi = read_openapi_spec(openapi_file)?;

    validate_openapi_kubernetes_extensions_exists(&openapi)?;

    let (
        kubernetes_operator_group,
        kubernetes_operator_version,
        kubernetes_operator_resource_ref,
        kubernetes_operator_include_tags,
        kubernetes_operator_metadata_spec_field_name,
    ) = extract_openapi_info(&openapi)?;

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

    let k8s_operator_dir = format!("{}/operator", base_path);
    let k8s_crdgen_dir = format!("{}/crdgen", base_path);
    let k8s_operator_types_dir = format!("{}/operator/src/types", base_path);
    let k8s_operator_controllers_dir = format!("{}/operator/src/controllers", base_path);
    let k8s_manifests_rbac_dir = format!("{}/manifests/rbac", base_path);
    let k8s_manifests_operator_dir = format!("{}/manifests/operator", base_path);
    let k8s_manifests_examples_dir = format!("{}/manifests/examples", base_path);

    if *all || (!*lib && !*manifests && !*controllers && !*types) {
        info!("Generating all...");
        generate_lib(&k8s_operator_dir)?;
        generate_types(
            &k8s_operator_types_dir,
            schemas.clone(),
            &kubernetes_operator_resource_ref,
        )?;
        let controllers = generate_controllers(
            base_path,
            &k8s_operator_controllers_dir,
            schemas.clone(),
            paths.clone(),
            kubernetes_operator_include_tags.clone(),
            kubernetes_operator_resource_ref.clone(),
        )?;
        generate_main_file(
            &k8s_operator_dir,
            &kubernetes_operator_group,
            &kubernetes_operator_version,
            controllers,
            schema_names.clone(),
        )?;
        generate_rbac_files(
            &k8s_manifests_rbac_dir,
            schema_names.clone(),
            &kubernetes_operator_group,
        )?;
        generate_operator_deployment_files(&k8s_manifests_operator_dir)?;
        generate_crdgen_file(&k8s_crdgen_dir, schema_names.clone())?;
        generate_examples(
            &k8s_manifests_examples_dir,
            &kubernetes_operator_metadata_spec_field_name,
            components.examples.into_iter().collect(),
            &kubernetes_operator_group,
            &kubernetes_operator_version,
            &kubernetes_operator_resource_ref.clone(),
        )?;
        return Ok(());
    }
    if *lib {
        info!("Generating lib...");
        generate_lib(&k8s_operator_dir)?;
    }
    if *manifests {
        info!("Generating manifests...");
        generate_rbac_files(
            &k8s_manifests_rbac_dir,
            schema_names.clone(),
            &kubernetes_operator_group,
        )?;
        generate_crdgen_file(&k8s_crdgen_dir, schema_names.clone())?;
        generate_examples(
            &k8s_manifests_examples_dir,
            &kubernetes_operator_metadata_spec_field_name,
            components.examples.into_iter().collect(),
            &kubernetes_operator_group,
            &kubernetes_operator_version,
            &kubernetes_operator_resource_ref.clone(),
        )?;
    }
    if *controllers {
        info!("Generating controllers...");
        let controllers = generate_controllers(
            base_path,
            &k8s_operator_controllers_dir,
            schemas.clone(),
            paths.clone(),
            kubernetes_operator_include_tags.clone(),
            kubernetes_operator_resource_ref.clone(),
        )?;
        generate_main_file(
            &k8s_operator_dir,
            &kubernetes_operator_group,
            &kubernetes_operator_version,
            controllers,
            schema_names.clone(),
        )?;
    }
    if *types {
        info!("Generating the types...");
        generate_types(
            &k8s_operator_types_dir,
            schemas.clone(),
            &kubernetes_operator_resource_ref,
        )?;
    }
    Ok(())
}

/// Generates RBAC files based on the provided resources and Kubernetes operator group.
fn generate_rbac_files(
    directory: &str,
    resources: Vec<String>,
    kubernetes_operator_group: &str,
) -> Result<(), AppError> {
    let base_path_rbac = Path::new(directory);

    generate_template_file(
        RoleTemplate {
            identifiers: RoleTemplateIdentifiers {
                api_group: kubernetes_operator_group.to_string(),
                resources: resources.clone(),
            },
        },
        base_path_rbac,
        "role.yaml",
    )?;
    generate_template_file(
        ClusterRoleTemplate {
            identifiers: ClusterRoleTemplateIdentifiers {
                api_group: kubernetes_operator_group.to_string(),
                resources: resources.clone(),
            },
        },
        base_path_rbac,
        "clusterrole.yaml",
    )?;
    generate_template_file(
        ServiceAccountTemplate {},
        base_path_rbac,
        "serviceaccount.yaml",
    )?;
    generate_template_file(RoleBindingTemplate {}, base_path_rbac, "rolebinding.yaml")?;
    generate_template_file(
        ClusterRoleBindingTemplate {},
        base_path_rbac,
        "clusterrolebinding.yaml",
    )?;

    Ok(())
}

/// Generates the operator deployment files.
fn generate_operator_deployment_files(directory: &str) -> Result<(), AppError> {
    let base_path_operator = Path::new(directory);

    generate_template_file(
        OperatorDeploymentTemplate {},
        base_path_operator,
        "deployment.yaml",
    )?;
    generate_template_file(OperatorSecretTemplate {}, base_path_operator, "secret.yaml")?;

    Ok(())
}

/// Generates the main file for the Kubernetes operator.
fn generate_main_file(
    directory: &str,
    api_group: &str,
    api_version: &str,
    mut controllers: Vec<String>,
    mut types: Vec<String>,
) -> Result<(), AppError> {
    let base_path = &Path::new(directory).join("src");
    let file_path = base_path.join("main.rs").to_string_lossy().to_string();
    if get_ignored_files()?.contains(&file_path) {
        return Ok(());
    }

    controllers.sort();

    types = types
        .iter()
        .map(|name| name.to_singular())
        .collect::<Vec<String>>();

    let base_path = &Path::new(directory).join("src");
    let content: String = MainTemplate {
        api_group: api_group.into(),
        api_version: api_version.into(),
        controllers,
        types,
    }
    .render()?;

    write_to_file(base_path, "main.rs", content)?;
    format_file(base_path.join("main.rs").to_str().unwrap())
}

/// Extracts controller attributes for a given operation.
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

/// Generates controllers based on the provided schemas, paths, and tags.
fn generate_controllers(
    working_dir: &str,
    directory: &str,
    schemas: HashMap<String, Schema>,
    paths: openapiv3::Paths,
    include_tags: Vec<String>,
    kubernetes_operator_resource_ref: String,
) -> Result<Vec<String>, AppError> {
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
            directory,
            schemas.clone(),
            tag.clone(),
            controller_attributes,
            kubernetes_operator_resource_ref.clone(),
        )?;

        if let Err(e) = upsert_line_to_file(
            format!("{}/.openapi-generator-ignore", working_dir).as_str(),
            format!("operator/src/controllers/{}.rs", tag.to_lowercase()).as_str(),
        ) {
            error!(
                "Failed to add controller to .openapi-generator-ignore file: {:?}",
                e
            );
        }
    }

    Ok(controllers.keys().cloned().collect())
}

/// Generates a controller based on the provided schemas, tag, and attributes.
fn generate_controller(
    directory: &str,
    schemas: HashMap<String, Schema>,
    tag: String,
    controller_attributes: &[ControllerAttributes],
    resource_remote_ref: String,
) -> Result<(), AppError> {
    if get_ignored_files()?.contains(&format!("{}/{}.rs", directory, tag.to_lowercase())) {
        return Ok(());
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

    let fields = get_fields_for_type(&schemas, &type_name, &resource_remote_ref)?;

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
    .render()?;

    let content_action_delete: String = ControllerActionDeleteTemplate {
        arg_name: tag.to_lowercase().to_singular(),
        kind_struct: type_name.clone(),
        controllers: controller_attributes.iter().collect(),
        resource_remote_ref: resource_remote_ref.clone(),
    }
    .render()?;

    let content_action_put: String = ControllerActionPutTemplate {
        arg_name: tag.to_lowercase().to_singular(),
        kind_struct: type_name.clone(),
        controllers: controller_attributes.iter().collect(),
        resource_remote_ref: resource_remote_ref.clone(),
    }
    .render()?;

    let content_action_post: String = ControllerActionPostTemplate {
        arg_name: tag.to_lowercase().to_singular(),
        kind_struct: type_name,
        controllers: controller_attributes.iter().collect(),
        resource_remote_ref: resource_remote_ref.clone(),
    }
    .render()?;

    content.push_str(&content_action_delete);
    content.push_str(&content_action_put);
    content.push_str(&content_action_post);

    let base_path: &Path = Path::new(directory);
    let file_name: String = format!("{}.rs", tag.to_lowercase());
    write_to_file(base_path, &file_name, content)?;
    format_file(base_path.join(file_name).to_str().unwrap())?;
    add_controller_to_modfile(directory, &tag.to_lowercase())?;
    Ok(())
}

/// Retrieves fields for a given schema type.
fn get_fields_for_type(
    schemas: &HashMap<String, Schema>,
    schema_name: &str,
    operator_resource_ref: &str,
) -> Result<Vec<Field>, AppError> {
    let schema = schemas
        .get(schema_name)
        .ok_or_else(|| AppError::Other(format!("Schema not found for type: {}", schema_name)))?;

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

/// Generates types based on the provided schemas and operator resource reference.
pub fn generate_types(
    directory: &str,
    schemas: HashMap<String, Schema>,
    operator_resource_ref: &str,
) -> Result<(), AppError> {
    for name in schemas.keys() {
        generate_type(
            schemas.clone(),
            name,
            "example.com",
            "v1",
            operator_resource_ref,
            directory,
        )?;
        add_type_to_modfile(name, directory)?;
    }

    Ok(())
}

/// Generates a type based on the provided schemas, name, and operator details.
fn generate_type(
    schemas: HashMap<String, Schema>,
    name: &str,
    operator_group: &str,
    operator_version: &str,
    operator_resource_ref: &str,
    directory: &str,
) -> Result<(), AppError> {
    if get_ignored_files()?.contains(&format!("{}/{}.rs", directory, name.to_lowercase())) {
        return Ok(());
    }

    let fields = match get_fields_for_type(&schemas, name, operator_resource_ref) {
        Ok(fields) => fields,
        Err(e) => {
            error!("Failed to get fields for type: {:?}", e);
            return Ok(());
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
    .render()?;

    let base_path: &Path = Path::new(directory);
    let file_name: String = format!("{}.rs", arg_name_clone);
    write_to_file(base_path, &file_name, content)?;
    format_file(base_path.join(file_name).to_str().unwrap())
}

fn generate_lib(directory: &str) -> Result<(), AppError> {
    let file_path = format!("{}/src/lib.rs", directory);
    if get_ignored_files()?.contains(&file_path) {
        return Ok(());
    }

    let content: String = LibTemplate {}.render()?;

    let base_path: &Path = &Path::new(directory).join("src");
    let file_name: String = "lib.rs".to_string();
    write_to_file(base_path, &file_name, content)?;
    format_file(base_path.join(file_name).to_str().unwrap())
}

/// Adds a type to the module file.
fn add_type_to_modfile(type_name: &str, directory: &str) -> Result<(), AppError> {
    let mod_file_path = Path::new(directory).join("mod.rs");

    if !mod_file_path.exists() {
        File::create(&mod_file_path).map_err(AppError::IoError)?;
    }

    upsert_line_to_file(
        mod_file_path.to_str().unwrap(),
        format!("pub mod {};", type_name.to_lowercase()).as_str(),
    )?;

    Ok(())
}

/// Adds a controller to the module file.
fn add_controller_to_modfile(directory: &str, controller_name: &str) -> Result<(), AppError> {
    let file_path = format!("{}/mod.rs", directory);
    match upsert_line_to_file(
        file_path.as_str(),
        format!("pub mod {};", controller_name.to_lowercase()).as_str(),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::Other(format!(
            "Failed to add controller to mod file: {:?}",
            e
        ))),
    }
}

/// Generates the CRD generator main file based on the provided resources.
fn generate_crdgen_file(directory: &str, resources: Vec<String>) -> Result<(), AppError> {
    let base_path: &Path = &Path::new(directory).join("src");
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
    let content = template.render()?;
    write_to_file(base_path, &file_name, content)?;
    format_file(base_path.join(file_name).to_str().unwrap())
}

/// Generates example manifests based on the provided examples.
fn generate_examples(
    directory: &str,
    kubernetes_operator_metadata_spec_field_name: &str,
    examples: std::collections::HashMap<String, ReferenceOr<openapiv3::Example>>,
    operator_group: &str,
    operator_version: &str,
    operator_resource_ref: &str,
) -> Result<(), AppError> {
    let examples_map: std::collections::HashMap<String, openapiv3::Example> = examples
        .into_iter()
        .filter_map(|(k, v)| match v {
            ReferenceOr::Item(item) => Some((k, item)),
            ReferenceOr::Reference { .. } => None,
        })
        .collect();
    for (name, example) in &examples_map {
        generate_manifest_from_example(
            directory,
            kubernetes_operator_metadata_spec_field_name,
            name,
            example,
            operator_group,
            operator_version,
            operator_resource_ref,
        )?;
    }

    Ok(())
}

/// Generates a manifest from an example.
fn generate_manifest_from_example(
    directory: &str,
    kubernetes_operator_metadata_spec_field_name: &str,
    name: &str,
    example: &openapiv3::Example,
    operator_group: &str,
    operator_version: &str,
    operator_resource_ref: &str,
) -> Result<(), AppError> {
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
        write_example_manifest(directory, name, resources)?;
    }

    Ok(())
}

/// Retrieves the metadata name from the provided map.
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

/// Generates a resource from the provided map.
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

/// Writes the example manifest to a file.
fn write_example_manifest(
    directory: &str,
    name: &str,
    resources: Vec<Resource>,
) -> Result<(), AppError> {
    let template = ExampleTemplate { resources };
    let base_path = Path::new(directory);
    let content = template.render()?;
    write_to_file(base_path, &format!("{}.yaml", name.to_lowercase()), content)
}
