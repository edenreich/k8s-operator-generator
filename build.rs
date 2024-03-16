use codegen::Scope;
use openapiv3::{OpenAPI, Schema};
use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use serde_yaml;
use std::fs::File;
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
    scope.import("kube", "CustomResource");
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
    scope.import("serde", "Deserialize");
    scope.import("serde_json", "json");
    scope.import("schemars", "JsonSchema");
    scope.import("k8s_openapi", "chrono");
    scope.import("k8s_openapi::api::core::v1", "Event");
    scope.import("k8s_openapi::api::core::v1", "EventSource");
    scope.import("k8s_openapi::api::core::v1", "ObjectReference");
    scope.import("k8s_openapi::apimachinery::pkg::apis::meta::v1", "Time");

    scope.raw("pub mod controllers;");

    // Generate a generic event handler function
    generate_generic_function(&mut scope);

    // Generate Rust structs for each schema in the components
    if let Some(components) = openapi.components.clone() {
        for (name, schema) in components.schemas {
            match schema {
                openapiv3::ReferenceOr::Reference { .. } => {
                    // Handle references here if needed
                }
                openapiv3::ReferenceOr::Item(item) => {
                    generate_struct(&mut scope, &name, "example.com", "v1", &item);
                    generate_function(&name);
                }
            }
        }
    }

    // Write the generated code to a file
    std::fs::write(dest, format!("{}\n", scope.to_string())).expect("Unable to write file");

    // Format the Rust code using rustfmt
    let output = Command::new("rustfmt")
        .arg(dest)
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
        "kube(group = \"{}\", version = \"{}\", kind = \"{}\", plural = \"{}\", namespaced)",
        api_group,
        api_version,
        name,
        name.to_lowercase() + "s"
    ));

    struct_.vis("pub");

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
                    struct_.field(field_name, &field_type);
                }
            }
        }
    }

    // Add the struct to the scope
    scope.push_struct(struct_);
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

    let function_string = quote! { #function }.to_string();
    let add_finalizer_function_string = quote! { #add_finalizer_function }.to_string();
    let remove_finalizer_function_string = quote! { #remove_finalizer_function }.to_string();
    let add_event_function_string = quote! { #add_event_function }.to_string();

    scope.raw(&function_string);
    scope.raw(&add_finalizer_function_string);
    scope.raw(&remove_finalizer_function_string);
    scope.raw(&add_event_function_string);
}

fn generate_function(name: &str) {
    let function_name = format_ident!("handle_{}", name.to_lowercase());
    let arg_name = format_ident!("{}", name.to_lowercase());
    let struct_name = format_ident!("{}", name);
    let create_function_name = format_ident!("{}s_post", name.to_lowercase());
    let list_function_name = format_ident!("{}s_get", name.to_lowercase());
    let get_function_name = format_ident!("{}s_id_get", name.to_lowercase());
    let update_function_name = format_ident!("{}s_id_put", name.to_lowercase());
    let delete_function_name = format_ident!("{}s_id_delete", name.to_lowercase());

    // Prepend the function with the use statements
    let mut file = format!(
        "use kube::Resource;
         use kube::api::WatchEvent;
         use kube::api::Api;
         use log::info;
         use log::error;
         use crate::add_finalizer;
         use crate::remove_finalizer;
         use crate::add_event;
         use crate::{};
         use openapi_client::models::{} as {}Dto;
         use openapi_client::apis::configuration::Configuration;
         use openapi_client::apis::default_api::{}s_get;
         use openapi_client::apis::default_api::{}s_post;
         use openapi_client::apis::default_api::{}s_id_get;
         use openapi_client::apis::default_api::{}s_id_delete;
         use openapi_client::apis::default_api::{}s_id_put;
         \n\n",
        struct_name, struct_name, struct_name, arg_name, arg_name, arg_name, arg_name, arg_name,
    );

    let function: ItemFn = parse_quote! {
        pub async fn #function_name(event: WatchEvent<#struct_name>, api: Api<#struct_name>) {
            let kind = #struct_name::kind(&());
            let kind_str = kind.to_string();

            let config = &Configuration {
                base_path: "http://localhost:8080".to_string(),
                user_agent: None,
                client: reqwest::Client::new(),
                basic_auth: todo!(),
                oauth_access_token: todo!(),
                bearer_access_token: todo!(),
                api_key: todo!(),
            };

            let (mut #arg_name, event_type) = match event {
                WatchEvent::Added(mut #arg_name) => {
                    if #arg_name.metadata.deletion_timestamp.is_none() {
                        add_finalizer(&mut #arg_name, api.clone()).await;
                        let dto = convert_to_dto(#arg_name);
                        #create_function_name(config, dto).await;
                    } else {
                        let dto = convert_to_dto(#arg_name);
                        if let Some(id) = dto.id {
                            #delete_function_name(config, id.as_str()).await;
                            remove_finalizer(&mut #arg_name, api.clone()).await;
                        } else {
                            error!("{} {} has no id", kind_str, #arg_name.metadata.name.clone().unwrap());
                        }
                    }
                    (#arg_name, "Added")
                }
                WatchEvent::Modified(mut #arg_name) => {
                    let dto = convert_to_dto(#arg_name);
                    if let Some(id) = dto.id {
                        #update_function_name(config, id.as_str(), dto).await;
                    } else {
                        error!("{} {} has no id", kind_str, #arg_name.metadata.name.clone().unwrap());
                    }
                    (#arg_name, "Modified")
                }
                WatchEvent::Deleted(mut #arg_name) => {
                    let dto = convert_to_dto(#arg_name);
                    if let Some(id) = dto.id {
                        #delete_function_name(config, id.as_str()).await;
                        remove_finalizer(&mut #arg_name, api.clone()).await;
                    } else {
                        error!("{} {} has no id", kind_str, #arg_name.metadata.name.clone().unwrap());
                    }
                    (#arg_name, "Deleted")
                }
                WatchEvent::Bookmark(bookmark) => {
                    info!("Cat Bookmark: {:?}", bookmark.metadata.resource_version);
                    return;
                }
                _ => {
                    info!("Cat Unknown event {:?}", event);
                    return;
                }
            };

            add_event(
                kind_str.clone(),
                &mut #arg_name,
                event_type.into(),
                kind_str.clone(),
                format!("Cat Resource {} Remotely", event_type),
            )
            .await;

            info!(
                "Cat {}: {:?} {:?}",
                event_type, #arg_name.metadata.name, #arg_name.metadata.finalizers
            );
        }
    };

    let dto = format_ident!("{}Dto", name);
    let resource = format_ident!("{}_resource", arg_name);

    let function_dto: TokenStream = quote! {
        fn convert_to_dto(#resource: #struct_name) -> #dto {
            todo!("Convert the resource to a DTO");
        }
    };

    // Convert the function into a string
    let function_string = quote! { #function }.to_string();
    let function_dto_string = quote! { #function_dto }.to_string();

    file.push_str(&function_string);
    file.push_str("\n\n");
    file.push_str(&function_dto_string);

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
