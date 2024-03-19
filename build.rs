use askama::Template;
use codegen::Scope;
use openapiv3::{OpenAPI, Schema};
use quote::quote;
use std::{
    fs::{DirBuilder, File, OpenOptions},
    io::{BufRead, BufReader, Error, Read, Write},
    process::Command,
};
use syn::{parse_quote, ItemFn};

const CONTROLLERS_DIR: &str = "src/controllers";
const TYPES_DIR: &str = "src/types";
const API_GROUP: &str = "example.com";

fn main() {
    let input = "openapi.yaml";
    let lib_file_path: String = "src/lib.rs".to_string();

    // Read the OpenAPI specification from the YAML file
    let mut file = File::open(input).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    // Parse the OpenAPI specification
    let openapi: OpenAPI = serde_yaml::from_str(&contents).expect("Unable to parse OpenAPI spec");

    // Generate lib.rs
    let mut scope = Scope::new();
    generate_lib_imports(&mut scope);
    generate_event_capturing_function(&mut scope);
    write_to_file(lib_file_path.clone(), scope.to_string());
    format_file(lib_file_path.clone());

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
                    let rust_code = generate_rust_code(&name, "v1", &item);
                    let mut file =
                        File::create(format!("{}/{}.rs", TYPES_DIR, name.to_lowercase())).unwrap();
                    write!(file, "{}", rust_code).unwrap();
                    Command::new("rustfmt")
                        .arg(format!("{}/{}.rs", TYPES_DIR, name.to_lowercase()))
                        .status()
                        .expect("Failed to run rustfmt on generated file");
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

fn generate_rust_code(name: &str, api_version: &str, schema: &Schema) -> String {
    let mut scope = Scope::new();
    generate_imports(&mut scope);
    generate_struct(&mut scope, name, api_version, schema);
    scope.to_string()
}

fn generate_imports(scope: &mut Scope) {
    scope.import("kube", "CustomResource");
    scope.import("serde", "Deserialize");
    scope.import("serde", "Serialize");
    scope.import("schemars", "JsonSchema");
}

fn generate_struct(scope: &mut Scope, name: &str, api_version: &str, schema: &Schema) {
    // Create a new struct with the given name
    let mut struct_ = codegen::Struct::new(&format!("{}Spec", name));

    // Add derive attributes to the struct
    struct_
        .derive("Debug")
        .derive("Default")
        .derive("Clone")
        .derive("Deserialize")
        .derive("Serialize")
        .derive("JsonSchema")
        .derive("CustomResource");

    struct_.attr(&format!(
        "kube(group = \"{}\", version = \"{}\", kind = \"{}\", plural = \"{}\", status = \"{}\", namespaced)",
        API_GROUP,
        api_version,
        name,
        name.to_lowercase() + "s",
        name.to_string() + "Status"
    ));
    struct_.vis("pub");

    let mut struct_status = codegen::Struct::new(&format!("{}Status", name));

    struct_status
        .derive("Debug")
        .derive("Default")
        .derive("Clone")
        .derive("Deserialize")
        .derive("Serialize")
        .derive("JsonSchema");

    struct_status.field("pub uuid", "Option<String>");
    struct_status.vis("pub");

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
                    struct_.field(format!("pub {}", field_name).as_str(), &field_type);
                }
            }
        }
    }

    // Add the struct to the scope
    scope.push_struct(struct_);
    scope.push_struct(struct_status);
}

fn generate_lib_imports(scope: &mut Scope) {
    scope.import("log", "error");
    scope.import("log", "info");
    scope.import("log", "debug");
    scope.import("kube", "Resource");
    scope.import("kube", "ResourceExt");
    scope.import("kube::core", "CustomResourceExt");
    scope.import("kube::api", "Api");
    scope.import("kube::api", "WatchEvent");
    scope.import("kube::api", "WatchParams");
    scope.import("kube::api", "Patch");
    scope.import("kube::api", "PatchParams");
    scope.import("kube::api", "PostParams");
    scope.import("kube::api", "ObjectMeta");
    scope.import("futures_util::stream", "StreamExt");
    scope.import("tokio::time", "sleep");
    scope.import("tokio::time", "Duration");
    scope.import("serde::de", "DeserializeOwned");
    scope.import("serde", "Serialize");
    scope.import("serde_json", "json");
    scope.import("k8s_openapi", "chrono");
    scope.import("k8s_openapi::api::core::v1", "Event");
    scope.import("k8s_openapi::api::core::v1", "EventSource");
    scope.import("k8s_openapi::api::core::v1", "ObjectReference");
    scope.import("k8s_openapi::apimachinery::pkg::apis::meta::v1", "Time");
    scope.import("openapi::apis::configuration", "Configuration");
    scope.import("std::sync", "Arc");

    scope.raw("pub mod types;");
    scope.raw("pub mod controllers;");
}

fn generate_event_capturing_function(scope: &mut Scope) {
    let function: ItemFn = parse_quote! {
        pub async fn watch_resource<T>(
            config: Arc<Configuration>,
            kubernetes_api: Api<T>,
            watch_params: WatchParams,
            handler: fn(Arc<Configuration>, WatchEvent<T>, Api<T>),
        ) -> anyhow::Result<()>
        where
            T: Clone + core::fmt::Debug + DeserializeOwned + 'static,
        {
            let mut stream = kubernetes_api.watch(&watch_params, "0").await?.boxed();
            loop {
                while let Some(event) = stream.next().await {
                    match event {
                        Ok(event) => handler(Arc::clone(&config), event, kubernetes_api.clone()),
                        Err(e) => error!("Error watching resource: {:?}", e),
                    }
                }
                sleep(Duration::from_secs(1)).await;
                stream = kubernetes_api.watch(&watch_params, "0").await?.boxed();
            }
        }
    };

    let add_finalizer_function: ItemFn = parse_quote! {
        pub async fn add_finalizer<T>(resource: &mut T, kubernetes_api: Api<T>)
        where
            T: Clone
                + Serialize
                + DeserializeOwned
                + Resource
                + CustomResourceExt
                + core::fmt::Debug
                + 'static,
        {
            let finalizer = String::from(format!("finalizers.{}", #API_GROUP));
            let finalizers = resource.meta_mut().finalizers.get_or_insert_with(Vec::new);
            if finalizers.contains(&finalizer) {
                debug!("Finalizer already exists");
                return;
            }
            finalizers.push(finalizer);

            let resource_name = resource.meta_mut().name.clone().unwrap();
            let resource_clone = resource.clone();
            let patch = Patch::Merge(&resource_clone);
            let patch_params = PatchParams {
                field_manager: resource.meta_mut().name.clone(),
                ..Default::default()
            };
            match kubernetes_api.patch(&resource_name, &patch_params, &patch).await {
                Ok(_) => debug!("Finalizer added successfully"),
                Err(e) => debug!("Failed to add finalizer: {:?}", e),
            };
        }
    };

    let remove_finalizer_function: ItemFn = parse_quote! {
        pub async fn remove_finalizer<T>(resource: &mut T, kubernetes_api: Api<T>)
            where
                T: Clone + Serialize + DeserializeOwned + Resource + CustomResourceExt + core::fmt::Debug + 'static {
            let finalizer = String::from(format!("finalizers.{}", #API_GROUP));
            if let Some(finalizers) = &mut resource.meta_mut().finalizers {
                if finalizers.contains(&finalizer) {
                    finalizers.retain(|f| f != &finalizer);
                    let patch = json ! ({ "metadata" : { "finalizers" : finalizers } });
                    let patch = Patch::Merge(&patch);
                    let patch_params = PatchParams {
                        field_manager: resource.meta_mut().name.clone(),
                        ..Default::default()
                    };
                    match kubernetes_api.patch(&resource.clone().meta_mut().name.clone().unwrap(), &patch_params, &patch).await {
                        Ok(_) => debug!("Finalizer removed successfully"),
                        Err(e) => debug!("Failed to remove finalizer: {:?}", e),
                    };
                }
            }
        }
    };

    let add_event_function: ItemFn = parse_quote! {
        pub async fn add_event<T>(kind: String, resource: &mut T, reason: &str, from: &str, message: &str)
        where
            T: CustomResourceExt
                + Clone
                + Serialize
                + DeserializeOwned
                + Resource
                + core::fmt::Debug
                + 'static,
        {
            let kube_client = kube::Client::try_default().await.unwrap();
            let namespace = resource.namespace().clone().unwrap_or_default();
            let kubernetes_api: Api<Event> = Api::namespaced(kube_client.clone(), &namespace);

            let resource_ref = ObjectReference {
                kind: Some(kind),
                namespace: resource.namespace().clone(),
                name: Some(resource.meta().name.clone().unwrap()),
                uid: resource.uid().clone(),
                ..Default::default()
            };

            let timestamp = chrono::Utc::now().to_rfc3339();
            let event_name = format!("{}-{}", resource.meta().name.clone().unwrap(), timestamp);

            let new_event = Event {
                metadata: ObjectMeta {
                    name: Some(event_name),
                    ..Default::default()
                },
                involved_object: resource_ref,
                reason: Some(reason.into()),
                message: Some(message.into()),
                type_: Some("Normal".into()),
                source: Some(EventSource {
                    component: Some(String::from(from)),
                    ..Default::default()
                }),
                first_timestamp: Some(Time(chrono::Utc::now())),
                last_timestamp: Some(Time(chrono::Utc::now())),
                ..Default::default()
            };

            match kubernetes_api.create(&PostParams::default(), &new_event).await {
                Ok(_) => debug!("Event added successfully"),
                Err(e) => debug!("Failed to add event: {:?}", e),
            };
        }
    };

    let change_status_function: ItemFn = parse_quote! {
        pub async fn change_status<T>(resource: &mut T, kubernetes_api: Api<T>, field: &str, value: String)
        where
            T: Clone + Serialize + DeserializeOwned + Resource + CustomResourceExt + core::fmt::Debug + 'static,
        {
            let name = resource.meta().name.clone().unwrap();
            let mut resource_json: serde_json::Value = serde_json::to_value(&resource).expect("Failed to serialize resource");
            resource_json["status"][field] = serde_json::json!(value);
            let new_resource: T = serde_json::from_value(resource_json).expect("Failed to deserialize resource");
            let resource_bytes = serde_json::to_vec(&new_resource).expect("Failed to serialize resource");
            match kubernetes_api.replace_status(&name, &PostParams::default(), resource_bytes).await {
                Ok(_) => info!("Status updated successfully for {}", name),
                Err(e) => info!("Failed to update status for {}: {:?}", name, e),
            };
        }
    };

    let function_string = quote! { #function }.to_string();
    let add_finalizer_function_string = quote! { #add_finalizer_function }.to_string();
    let remove_finalizer_function_string = quote! { #remove_finalizer_function }.to_string();
    let add_event_function_string = quote! { #add_event_function }.to_string();
    let change_status_function_string = quote! { #change_status_function }.to_string();

    scope.raw(&function_string);
    scope.raw(&add_finalizer_function_string);
    scope.raw(&remove_finalizer_function_string);
    scope.raw(&add_event_function_string);
    scope.raw(&change_status_function_string);
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

struct Identifiers<'a> {
    tag_name: &'a str,
    arg_name: &'a str,
    type_name: &'a str,
}

#[derive(Template)]
#[template(path = "controller.jinja")]
struct ControllerTemplate<'a> {
    identifiers: Identifiers<'a>,
}

fn generate_controller(name: &str) {
    let name_singular = convert_to_singular(name);
    let content: String = ControllerTemplate {
        identifiers: Identifiers {
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
