use askama::Template;
use openapiv3::{OpenAPI, Schema};
use std::{
    fs::{DirBuilder, File, OpenOptions},
    io::{BufRead, BufReader, Error, Read, Write},
    process::Command,
};

const CONTROLLERS_DIR: &str = "src/controllers";
const TYPES_DIR: &str = "src/types";
const LIB_FILEPATH: &str = "src/lib.rs";
const API_GROUP: &str = "example.com";

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

    // Create the types directory if it doesn't exist
    DirBuilder::new()
        .recursive(true)
        .create(TYPES_DIR)
        .expect("Unable to create types directory");

    // Generate Rust code for each schema in the components
    if let Some(components) = openapi.components.clone() {
        for (name, schema) in components.schemas {
            match schema {
                openapiv3::ReferenceOr::Reference { .. } => {
                    // Handle references here if needed
                }
                openapiv3::ReferenceOr::Item(item) => {
                    generate_type(&name, "v1", &item);
                }
            }
        }
    }

    // Generate Rust code for each operation in the paths
    empty_file(format!("{}/mod.rs", CONTROLLERS_DIR))
        .expect("Unable to empty controllers mod.rs file");
    for (_, path_item) in openapi.paths.iter() {
        let path_item = match path_item {
            openapiv3::ReferenceOr::Item(i) => i,
            openapiv3::ReferenceOr::Reference { .. } => {
                // Handle the case where the path item is a reference, not a direct item
                continue;
            }
        };

        // Iterate over the operations of each path
        let operations = [
            &path_item.get,
            &path_item.put,
            &path_item.post,
            &path_item.delete,
            &path_item.options,
            &path_item.head,
            &path_item.patch,
            &path_item.trace,
        ];
        for operation in operations.iter().filter_map(|o| o.as_ref()) {
            // Generate a controller for each tag
            for tag in &operation.tags {
                let controller_name = format!("{}", tag);
                generate_controller(&controller_name);
                add_controller_to_modfile(&controller_name)
                    .expect("Unable to add controller to mod file");
            }
        }
    }

    let components = openapi
        .components
        .clone()
        .expect("No components in OpenAPI spec");
    let schema_names = components
        .schemas
        .keys()
        .map(|name| name.to_string())
        .collect::<Vec<String>>();

    // Generate a mod.rs file that publicly exports all the generated modules
    for name in schema_names.clone() {
        add_type_to_modfile(&name).expect("Unable to add type to mod file");
    }

    // Generate the RBAC for the operator
    let mut resources = String::new();
    for schema_name in schema_names.clone() {
        resources.push_str(&format!("      - {}\n", schema_name.to_lowercase() + "s"));
    }
    resources.pop();

    let role_file_content = get_role_file_content(&resources);
    write_to_file("manifests/rbac/role.yaml".to_string(), role_file_content);

    let cluster_role_file_content = get_cluster_role_file_content(&resources);
    write_to_file(
        "manifests/rbac/clusterrole.yaml".to_string(),
        cluster_role_file_content,
    );

    let service_account_file_content = get_service_account_file_content();
    write_to_file(
        "manifests/rbac/serviceaccount.yaml".to_string(),
        service_account_file_content,
    );

    let role_binding_file_content = get_role_binding_file_content();
    write_to_file(
        "manifests/rbac/rolebinding.yaml".to_string(),
        role_binding_file_content,
    );

    let cluster_role_binding_file_content = get_cluster_role_binding_file_content();
    write_to_file(
        "manifests/rbac/clusterrolebinding.yaml".to_string(),
        cluster_role_binding_file_content,
    );

    // Generate the operator deployment
    let deployment_file_content = get_operator_deployment_file_content();
    write_to_file(
        "manifests/operator/deployment.yaml".to_string(),
        deployment_file_content,
    );

    // Generate the code that generates the CRDs
    let crdgen_file_content = get_crdgen_file_content(&schema_names);
    write_to_file("src/crdgen.rs".to_string(), crdgen_file_content);
    format_file("src/crdgen.rs".to_string());

    // Generate examples from OAS
    for (name, example) in components.examples {
        if let openapiv3::ReferenceOr::Item(example) = example {
            let manifest = generate_manifest_from_example(&name, &example);

            write_to_file(
                format!("manifests/examples/{}.yaml", name.to_lowercase()),
                manifest,
            )
        }
    }
}

struct Field {
    pub_name: String,
    field_type: String,
}

struct TypeIdentifiers {
    tag_name: String,
    type_name: String,
    api_version: String,
    group_name: String,
    fields: Vec<Field>,
}

#[derive(Template)]
#[template(path = "type.jinja")]
struct TypeTemplate {
    identifiers: TypeIdentifiers,
}

fn generate_type(name: &str, api_version: &str, schema: &Schema) {
    let mut fields = Vec::new();

    // Add fields to the struct based on the schema
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

    let tag_name = name.to_string();
    let arg_name = name.to_lowercase();
    let type_name = uppercase_first_letter(name);
    let arg_name_clone = arg_name.clone();

    let content: String = TypeTemplate {
        identifiers: TypeIdentifiers {
            tag_name,
            type_name,
            api_version: api_version.to_string(),
            group_name: API_GROUP.to_string(),
            fields,
        },
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

struct ControllerIdentifiers<'a> {
    tag_name: &'a str,
    arg_name: &'a str,
    type_name: &'a str,
}

#[derive(Template)]
#[template(path = "controller.jinja")]
struct ControllerTemplate<'a> {
    identifiers: ControllerIdentifiers<'a>,
}

fn generate_controller(name: &str) {
    let name_singular = convert_to_singular(name);
    let content: String = ControllerTemplate {
        identifiers: ControllerIdentifiers {
            tag_name: name,
            arg_name: name_singular.clone().as_str(),
            type_name: &uppercase_first_letter(name_singular.clone().as_str()),
        },
    }
    .render()
    .unwrap();
    let file_path = format!("{}/{}.rs", CONTROLLERS_DIR, name.to_lowercase());
    write_to_file(file_path.to_owned(), content);
    format_file(file_path)
}

#[derive(serde::Serialize)]
struct K8sManifest {
    #[serde(rename = "apiVersion")]
    api_version: String,
    kind: String,
    metadata: Metadata,
    spec: serde_json::Value,
}

#[derive(serde::Serialize)]
struct Metadata {
    name: String,
    namespace: String,
}

fn generate_manifest_from_example(name: &str, example: &openapiv3::Example) -> String {
    let mut manifest = String::from("---\n");
    if let Some(mut value) = example.value.clone() {
        if value.is_object() {
            let obj = value.as_object_mut().unwrap();
            obj.remove("uuid");
        }
        let k8s_manifest = K8sManifest {
            api_version: format!("{}/v1", API_GROUP),
            kind: name.to_string(),
            metadata: Metadata {
                name: "example".to_string(),
                namespace: "default".to_string(),
            },
            spec: value,
        };
        let yaml_str = serde_yaml::to_string(&k8s_manifest).unwrap();
        manifest.push_str(&yaml_str);
    }
    manifest
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

    std::fs::write(file_path, file_content).expect("Unable to write file");
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

fn convert_to_singular(name: &str) -> String {
    if name == "Kubernetes" {
        return name.to_string();
    } else if name.ends_with("ies") {
        let mut s = name.to_string();
        s.truncate(s.len() - 3);
        s.push('y');
        return s;
    } else if name.ends_with("s") {
        let mut s = name.to_string();
        s.pop();
        return s;
    }
    name.to_string()
}

fn uppercase_first_letter(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// File templates
///
/// Later will be moved to a separate template files
///
fn get_role_file_content(resources: &str) -> String {
    format!(
        r#"---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: operator-role # Give this a meaningful name
rules:
  - apiGroups:
      - {}
    resources:
{}
    verbs:
      - get
      - list
      - watch
      - create
      - update
      - patch
      - delete
  - apiGroups:
      - ''
    resources:
      - events
    verbs:
      - create
      - patch
"#,
        API_GROUP, resources
    )
}

fn get_cluster_role_file_content(resources: &str) -> String {
    format!(
        r#"---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: operator-cluster-role
rules:
  - apiGroups:
      - {}
    resources:
{}
    verbs:
      - get
      - list
      - watch
      - create
      - update
      - patch
      - delete
"#,
        API_GROUP, resources
    )
}

fn get_service_account_file_content() -> String {
    format!(
        r#"---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: operator-service-account
"#,
    )
}

fn get_role_binding_file_content() -> String {
    format!(
        r#"---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: operator-role-binding
subjects:
  - kind: ServiceAccount
    name: operator-service-account
roleRef:
  kind: Role
  name: operator-role
  apiGroup: rbac.authorization.k8s.io
"#,
    )
}

fn get_cluster_role_binding_file_content() -> String {
    format!(
        r#"---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: operator-cluster-role-binding
subjects:
  - kind: ServiceAccount
    name: operator-service-account
    namespace: default
roleRef:
  kind: ClusterRole
  name: operator-cluster-role
  apiGroup: rbac.authorization.k8s.io
"#,
    )
}

fn get_operator_deployment_file_content() -> String {
    format!(
        r#"---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: operator-deployment
  labels:
    app: operator
spec:
  replicas: 1
  selector:
    matchLabels:
      app: operator
  template:
    metadata:
      labels:
        app: operator
    spec:
      serviceAccountName: operator-service-account
      containers:
        - name: operator
          image: operator:latest
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 512Mi
"#,
    )
}

fn get_crdgen_file_content(schema_names: &Vec<String>) -> String {
    let mut insert_lines = String::new();
    for schema_name in schema_names.clone() {
        let line = format!(
            "k8s_operator::types::{}::{}::crd(),\n",
            schema_name.to_lowercase(),
            schema_name,
        );
        insert_lines.push_str(&line);
    }

    format!(
        r#"
use kube::CustomResourceExt;

fn main() {{
    let crds = vec![
        {}
    ];

    for crd in crds {{
        match serde_yaml::to_string(&crd) {{
            Ok(yaml) => print!("---\n{{}}", yaml),
            Err(e) => eprintln!("Error serializing CRD to YAML: {{}}", e),
        }}
    }}
}}
"#,
        insert_lines
    )
}
