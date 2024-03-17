use codegen::Scope;
use openapiv3::{OpenAPI, Schema};
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use serde_yaml;
use std::fs::DirBuilder;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};
use std::process::Command;
use syn::parse_quote;
use syn::ItemFn;

fn main() {
    let input = "./openapi.yaml";
    let dest = "./src/lib.rs";

    // Read the OpenAPI specification from the YAML file
    let mut file = File::open(input).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    // Parse the OpenAPI specification
    let openapi: OpenAPI = serde_yaml::from_str(&contents).expect("Unable to parse OpenAPI spec");

    // Create a new codegen scope
    let mut scope = Scope::new();

    // Add necessary imports to the scope
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

    scope.raw("pub mod types;");
    scope.raw("pub mod controllers;");

    // Generate a generic event handler function
    generate_generic_function(&mut scope);

    // Write the Rust code to a file
    let mut file = File::create(dest).unwrap();
    write!(file, "{}", scope.to_string()).unwrap();

    // Run rust fmt on the generated file
    Command::new("rustfmt")
        .arg(dest)
        .status()
        .expect("Failed to run rustfmt on generated file");

    // Create the types directory if it doesn't exist
    DirBuilder::new()
        .recursive(true)
        .create("src/types")
        .unwrap();

    // Generate Rust code for each schema in the components
    if let Some(components) = openapi.components.clone() {
        for (name, schema) in components.schemas {
            match schema {
                openapiv3::ReferenceOr::Reference { .. } => {
                    // Handle references here if needed
                }
                openapiv3::ReferenceOr::Item(item) => {
                    let rust_code = generate_rust_code(&name, "example.com", "v1", &item);
                    // Write the Rust code to a file in the types directory
                    let mut file =
                        File::create(format!("src/types/{}.rs", name.to_lowercase())).unwrap();
                    write!(file, "{}", rust_code).unwrap();

                    // Run rust fmt on the generated file
                    Command::new("rustfmt")
                        .arg(format!("src/types/{}.rs", name.to_lowercase()))
                        .status()
                        .expect("Failed to run rustfmt on generated file");
                }
            }
        }
    }

    // Generate a mod.rs file that publicly exports all the generated modules
    let mut mod_file = File::create("src/types/mod.rs").unwrap();
    if let Some(components) = openapi.components.clone() {
        for name in components.schemas.keys() {
            writeln!(mod_file, "pub mod {};", name.to_lowercase()).unwrap();
        }
    }

    // Generate a mod.rs file that publicly exports all the generated modules
    let mut mod_file = File::create("src/controllers/mod.rs").unwrap();
    if let Some(components) = openapi.components.clone() {
        for name in components.schemas.keys() {
            writeln!(mod_file, "pub mod {};", name.to_lowercase()).unwrap();
        }
    }

    let schema_names = openapi
        .components
        .unwrap()
        .schemas
        .keys()
        .map(|name| name.to_string())
        .collect::<Vec<String>>();

    // Generate the Rust code
    let mut insert_lines = String::new();
    for schema_name in schema_names {
        let line = format!(
            "serde_yaml::to_string(&k8s_operator::types::{}::{}::crd()).unwrap(),\n",
            schema_name.to_lowercase(),
            schema_name,
        );
        insert_lines.push_str(&line);
    }
    let mut crdgen_file = File::create("src/crdgen.rs").unwrap();
    write!(
        crdgen_file,
        r#"
use kube::CustomResourceExt;

fn main() {{
    print!(
        "---\n{{}}\n---\n{{}}",
        {}
    );
}}
"#,
        insert_lines
    )
    .unwrap();

    // format the file using rustfmt
    Command::new("rustfmt")
        .arg("src/crdgen.rs")
        .status()
        .expect("Failed to run rustfmt on src/crdgen.rs");
}

fn generate_rust_code(name: &str, api_group: &str, api_version: &str, schema: &Schema) -> String {
    let mut scope = Scope::new();
    generate_imports(&mut scope);
    generate_struct(&mut scope, name, api_group, api_version, schema);
    generate_controller(name);
    scope.to_string()
}

fn generate_imports(scope: &mut Scope) {
    scope.import("kube", "CustomResource");
    scope.import("serde", "Deserialize");
    scope.import("serde", "Serialize");
    scope.import("schemars", "JsonSchema");
}

fn generate_struct(
    scope: &mut Scope,
    name: &str,
    api_group: &str,
    api_version: &str,
    schema: &Schema,
) {
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
        api_group,
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

fn generate_generic_function(scope: &mut Scope) {
    let function: ItemFn = parse_quote! {
        pub async fn watch_resource<T>(
            api: Api<T>,
            watch_params: WatchParams,
            handler: fn(WatchEvent<T>, Api<T>),
        ) -> anyhow::Result<()>
        where
            T: Clone + core::fmt::Debug + DeserializeOwned + 'static,
        {
            let mut stream = api.watch(&watch_params, "0").await?.boxed();

            loop {
                while let Some(event) = stream.next().await {
                    match event {
                        Ok(event) => handler(event, api.clone()),
                        Err(e) => error!("Error watching resource: {:?}", e),
                    }
                }

                sleep(Duration::from_secs(1)).await;
                stream = api.watch(&watch_params, "0").await?.boxed();
            }
        }
    };

    let add_finalizer_function: ItemFn = parse_quote! {
        pub async fn add_finalizer<T>(resource: &mut T, api: Api<T>)
        where
            T: Clone
                + Serialize
                + DeserializeOwned
                + Resource
                + CustomResourceExt
                + core::fmt::Debug
                + 'static,
        {
            let finalizer = String::from("finalizers.example.com");
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
            match api.patch(&resource_name, &patch_params, &patch).await {
                Ok(_) => debug!("Finalizer added successfully"),
                Err(e) => debug!("Failed to add finalizer: {:?}", e),
            };
        }
    };

    let remove_finalizer_function: ItemFn = parse_quote! {
        pub async fn remove_finalizer<T>(resource: &mut T, api: Api<T>)
            where
                T: Clone + Serialize + DeserializeOwned + Resource + CustomResourceExt + core::fmt::Debug + 'static {
            let finalizer = String::from("finalizers.example.com");
            if let Some(finalizers) = &mut resource.meta_mut().finalizers {
                if finalizers.contains(&finalizer) {
                    finalizers.retain(|f| f != &finalizer);
                    let patch = json ! ({ "metadata" : { "finalizers" : finalizers } });
                    let patch = Patch::Merge(&patch);
                    let patch_params = PatchParams {
                        field_manager: resource.meta_mut().name.clone(),
                        ..Default::default()
                    };
                    match api.patch(&resource.clone().meta_mut().name.clone().unwrap(), &patch_params, &patch).await {
                        Ok(_) => debug!("Finalizer removed successfully"),
                        Err(e) => debug!("Failed to remove finalizer: {:?}", e),
                    };
                }
            }
        }
    };

    let add_event_function: ItemFn = parse_quote! {
        pub async fn add_event<T>(kind: String, resource: &mut T, reason: String, from: String, message: String)
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
            let api: Api<Event> = Api::namespaced(kube_client.clone(), &namespace);

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
                    component: Some(from),
                    ..Default::default()
                }),
                first_timestamp: Some(Time(chrono::Utc::now())),
                last_timestamp: Some(Time(chrono::Utc::now())),
                ..Default::default()
            };

            match api.create(&PostParams::default(), &new_event).await {
                Ok(_) => debug!("Event added successfully"),
                Err(e) => debug!("Failed to add event: {:?}", e),
            };
        }
    };

    let change_status_function: ItemFn = parse_quote! {
        pub async fn change_status<T>(resource: &mut T, api: Api<T>, field: &str, value: String)
        where
            T: Clone + Serialize + DeserializeOwned + Resource + CustomResourceExt + core::fmt::Debug + 'static,
        {
            let name = resource.meta().name.clone().unwrap();
            let mut resource_json: serde_json::Value = serde_json::to_value(&resource).expect("Failed to serialize resource");
            resource_json["status"][field] = serde_json::json!(value);
            let new_resource: T = serde_json::from_value(resource_json).expect("Failed to deserialize resource");
            let resource_bytes = serde_json::to_vec(&new_resource).expect("Failed to serialize resource");
            match api.replace_status(&name, &PostParams::default(), resource_bytes).await {
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

fn generate_controller(name: &str) {
    let function_name = format_ident!("handle_{}", name.to_lowercase());
    let arg_name = format_ident!("{}", name.to_lowercase());
    let struct_name = format_ident!("{}", name);
    let _status_name = format_ident!("{}Status", name);
    let create_function_name = format_ident!("{}s_post", name.to_lowercase());
    let _list_function_name = format_ident!("{}s_get", name.to_lowercase());
    let _get_function_name = format_ident!("{}s_id_get", name.to_lowercase());
    let update_function_name = format_ident!("{}s_id_put", name.to_lowercase());
    let delete_function_name = format_ident!("{}s_id_delete", name.to_lowercase());

    // Prepend the function with the use statements
    let mut file = format!(
        "use kube::Resource;
         use kube::api::WatchEvent;
         use kube::api::Api;
         use log::info;
         use log::error;
         use log::warn;
         use crate::add_finalizer;
         use crate::remove_finalizer;
         use crate::add_event;
         use crate::change_status;
         use crate::types::{}::{};
         use openapi_client::models::{} as {}Dto;
         use openapi_client::apis::configuration::Configuration;
         use openapi_client::apis::default_api::{}s_post;
         use openapi_client::apis::default_api::{}s_id_delete;
         use openapi_client::apis::default_api::{}s_id_put;
         \n\n",
        struct_name.to_string().to_lowercase(),
        struct_name,
        struct_name,
        struct_name,
        arg_name,
        arg_name,
        arg_name
    );

    let main_handler: ItemFn = parse_quote! {
        pub async fn #function_name(event: WatchEvent<#struct_name>, api: Api<#struct_name>) {
            let kind = #struct_name::kind(&());
            let kind_str = kind.to_string();

            let config = &Configuration {
                base_path: "http://localhost:8080".to_string(),
                user_agent: None,
                client: reqwest::Client::new(),
                ..Configuration::default()
            };

            match event {
                WatchEvent::Added(mut #arg_name) => handle_added(config, kind_str, &mut #arg_name, api).await,
                WatchEvent::Modified(mut #arg_name) => handle_modified(config, kind_str, &mut #arg_name, api).await,
                // WatchEvent::Deleted(mut #arg_name) => handle_deleted(config, kind_str, &mut #arg_name, api).await,
                WatchEvent::Bookmark(bookmark) => {
                    info!("Cat Bookmark: {:?}", bookmark.metadata.resource_version);
                    return;
                }
                _ => {
                    info!("Cat Unknown event {:?}", event);
                    return;
                }
            };
        }
    };

    let dto = format_ident!("{}Dto", name);
    let resource = format_ident!("{}_resource", arg_name);

    let function_dto: TokenStream = quote! {
        fn convert_to_dto(#resource: #struct_name) -> #dto {
            let _uuid = match #resource.status {
                Some(status) => status.uuid,
                None => None,
            };
            todo!("Convert the resource to a DTO");
        }
    };

    let handle_added: ItemFn = parse_quote! {
        async fn handle_added(config: &Configuration, kind_str: String, #arg_name: &mut #struct_name, api: Api<#struct_name>) {
            if #arg_name.metadata.deletion_timestamp.is_some() {
                handle_deleted(config, kind_str, #arg_name, api).await;
                return;
            }

            if #arg_name.status.is_none() {
                #arg_name.status = Some(Default::default());
            }

            let model = #arg_name.clone();
            let name = #arg_name.metadata.name.clone().unwrap();
            let dto = convert_to_dto(model);

            if dto.uuid.is_some() {
                info!(
                    "{} {} already exists",
                    kind_str,
                    name
                );
                // Todo - check drift
                return;
            }
            add_finalizer(#arg_name, api.clone()).await;
            match #create_function_name(config, dto).await {
                Ok(resp) => {
                    info!("{} {} created successfully", kind_str, name);
                    add_event(
                        kind_str.clone(),
                        #arg_name,
                        "Normal".into(),
                        kind_str.clone(),
                        format!("{} {} created remotely", kind_str, name),
                    ).await;

                    if let (Some(_status), Some(uuid)) = (#arg_name.status.as_mut(), resp.uuid.clone()) {
                        change_status(#arg_name, api.clone(), "uuid", uuid).await;
                    } else {
                        warn!("Failed to retrieve uuid from response");
                    }
                }
                Err(e) => {
                    error!("Failed to add {} {}: {:?}", kind_str, name, e);
                    add_event(
                        kind_str.clone(),
                        #arg_name,
                        "Error".into(),
                        kind_str.clone(),
                        format!("Failed to create {} {} remotely", kind_str, name),
                    ).await;
                }
            }
        }
    };

    let handle_modified: ItemFn = parse_quote! {
        async fn handle_modified(config: &Configuration, kind_str: String, #arg_name: &mut #struct_name, api: Api<#struct_name>) {
            if #arg_name.metadata.deletion_timestamp.is_some() {
                handle_deleted(config, kind_str, #arg_name, api).await;
                return;
            }

            let model = #arg_name.clone();
            let dto = convert_to_dto(model);
            let name = #arg_name.metadata.name.clone().unwrap();

            if let Some(ref uuid) = dto.uuid.clone() {
                match #update_function_name(config, uuid.as_str(), dto).await {
                    Ok(_) => {
                        info!("{} {} modified successfully", kind_str, name);
                        add_event(
                            kind_str.clone(),
                            #arg_name,
                            "Normal".into(),
                            kind_str.clone(),
                            format!("{} {} modified remotely", kind_str, name),
                        ).await;
                    }
                    Err(e) => {
                        error!("Failed to update {} {}: {:?}", kind_str, name, e);
                        add_event(
                            kind_str.clone(),
                            #arg_name,
                            "Error".into(),
                            kind_str.clone(),
                            format!("Failed to update {} {} remotely", kind_str, name),
                        ).await;
                    }
                }
            } else {
                error!(
                    "{} {} has no id",
                    kind_str,
                    name,
                );
                add_event(
                    kind_str.clone(),
                    #arg_name,
                    "Error".into(),
                    kind_str.clone(),
                    format!("Failed to update {} {}", kind_str, name),
                ).await;
            }
        }
    };

    let handle_deleted: ItemFn = parse_quote! {
        async fn handle_deleted(config: &Configuration, kind_str: String, #arg_name: &mut #struct_name, api: Api<#struct_name>) {
            let model = #arg_name.clone();
            let dto = convert_to_dto(model);
            let name = #arg_name.metadata.name.clone().unwrap();

            if let Some(uuid) = dto.uuid.clone() {
                match #delete_function_name(config, uuid.as_str()).await {
                    Ok(_res) => {
                        info!("{} {} deleted successfully", kind_str, name);
                        add_event(
                            kind_str.clone(),
                            #arg_name,
                            "Normal".into(),
                            kind_str.clone(),
                            format!("{} {} deleted remotely", kind_str, name),
                        ).await;
                        remove_finalizer(#arg_name, api.clone()).await;
                    }
                    Err(e) => {
                        error!("Failed to delete {} {}: {:?}", kind_str, name, e);
                        add_event(
                            kind_str.clone(),
                            #arg_name,
                            "Error".into(),
                            kind_str.clone(),
                            format!("Failed to delete {} {} remotely", kind_str, name),
                        ).await;
                    }
                }
            } else {
                error!(
                    "{} {} has no id",
                    kind_str,
                    #arg_name.metadata.name.clone().unwrap()
                );
                add_event(
                    kind_str.clone(),
                    #arg_name,
                    "Error".into(),
                    kind_str.clone(),
                    format!("Failed to delete {} {}", kind_str, name),
                )
                .await;
            }
        }
    };

    // Convert the function into a string
    let function_dto_string = quote! { #function_dto }.to_string();
    let main_handler_string = quote! { #main_handler }.to_string();
    let handle_added_string = quote! { #handle_added }.to_string();
    let handle_modified_string = quote! { #handle_modified }.to_string();
    let handle_deleted_string = quote! { #handle_deleted }.to_string();

    file.push_str(&function_dto_string);
    file.push_str("\n\n");
    file.push_str(&main_handler_string);
    file.push_str("\n\n");
    file.push_str(&handle_added_string);
    file.push_str("\n\n");
    file.push_str(&handle_modified_string);
    file.push_str("\n\n");
    file.push_str(&handle_deleted_string);

    // Write the function to a new file
    let file_path = format!("src/controllers/{}.rs", name.to_lowercase());

    // Check if the file_path is in the .openapi-generator-ignore file
    let ignore_file =
        std::fs::File::open(".openapi-generator-ignore").expect("Unable to open file");
    let reader = BufReader::new(ignore_file);
    let ignored_files: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    if !ignored_files.contains(&file_path) {
        let file_path_clone = file_path.clone();
        std::fs::write(file_path, file).expect("Unable to write file");

        // Format the Rust code using rustfmt
        let output = Command::new("rustfmt")
            .arg(file_path_clone)
            .output()
            .expect("Failed to execute command");

        // Check the output of the rustfmt command
        if !output.status.success() {
            eprintln!(
                "rustfmt failed with output:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
