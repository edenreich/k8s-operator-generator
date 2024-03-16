use codegen::Scope;
use openapiv3::{OpenAPI, Schema};
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
    scope.import("kube::api", "Api");
    scope.import("kube::api", "WatchEvent");
    scope.import("kube::api", "WatchParams");
    scope.import("kube::api", "Patch");
    scope.import("kube::api", "PatchParams");
    scope.import("kube", "CustomResource");
    scope.import("kube::core", "CustomResourceExt");
    scope.import("kube::core", "Resource");
    scope.import("log", "error");
    scope.import("log", "info");
    scope.import("log", "debug");
    scope.import("futures_util::stream", "StreamExt");
    scope.import("tokio::time", "sleep");
    scope.import("tokio::time", "Duration");
    scope.import("serde::de", "DeserializeOwned");
    scope.import("serde", "Serialize");
    scope.import("serde", "Deserialize");
    scope.import("serde_json", "json");
    scope.import("schemars", "JsonSchema");

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
                        openapiv3::SchemaKind::Type(openapiv3::Type::Integer(_)) => "u32",
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

    let function_string = quote! { #function }.to_string();
    let add_finalizer_function_string = quote! { #add_finalizer_function }.to_string();
    let remove_finalizer_function_string = quote! { #remove_finalizer_function }.to_string();

    scope.raw(&function_string);
    scope.raw(&add_finalizer_function_string);
    scope.raw(&remove_finalizer_function_string);
}

fn generate_function(name: &str) {
    let function_name = format_ident!("handle_{}", name.to_lowercase());
    let arg_name = format_ident!("{}", name.to_lowercase());
    let struct_name = format_ident!("{}", name);
    let struct_name_string = struct_name.to_string();

    let function: ItemFn = parse_quote! {
        pub async fn #function_name(event: WatchEvent<#struct_name>, api: Api<#struct_name>) {
            match event {
                WatchEvent::Added(mut #arg_name) => {
                    if #arg_name.metadata.deletion_timestamp.is_some() {
                        info!(
                            "{} Sending API call to delete the remote resource and wait for response: {:?}",
                            #struct_name_string, #arg_name.metadata.name
                        );
                        remove_finalizer(&mut #arg_name, api.clone()).await;
                    } else {
                        add_finalizer(&mut #arg_name, api.clone()).await;
                        info!(
                            "{} Added: {:?} {:?}",
                            #struct_name_string, #arg_name.metadata.name, #arg_name.metadata.finalizers
                        )
                    }
                }
                WatchEvent::Modified(#arg_name) => {
                    info!(
                        "{} Modified: {:?} {:?}",
                        #struct_name_string, #arg_name.metadata.name, #arg_name.metadata.finalizers
                    );
                }
                WatchEvent::Deleted(#arg_name) => {
                    info!(
                        "{} Deleted: {:?} {:?}",
                        #struct_name_string, #arg_name.metadata.name, #arg_name.metadata.finalizers
                    );
                }
                _ => {
                    info!("{} Unknown event", #struct_name_string);
                }
            }
        }

    };

    // Convert the function into a string
    let function_string = quote! { #function }.to_string();

    // Prepend the function with the use statements
    let function_string = format!(
        "use kube::api::{};\nuse log::info;use crate::{};use crate::{};\n\n{}",
        "{WatchEvent, Api}", name, "{add_finalizer, remove_finalizer}", function_string
    );

    // Write the function to a new file
    let file_path = format!("src/controllers/{}.rs", name.to_lowercase());

    // Check if the file_path is in the .openapi-generator-ignore file
    let ignore_file =
        std::fs::File::open(".openapi-generator-ignore").expect("Unable to open file");
    let reader = BufReader::new(ignore_file);
    let ignored_files: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    if !ignored_files.contains(&file_path) {
        let file_path_clone = file_path.clone();
        std::fs::write(file_path, function_string).expect("Unable to write file");

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
