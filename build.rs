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
    scope.import("log", "error");
    scope.import("futures_util::stream", "StreamExt");
    scope.import("tokio::time", "sleep");
    scope.import("tokio::time", "Duration");

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
        .derive("serde::Deserialize")
        .derive("serde::Serialize")
        .derive("schemars::JsonSchema")
        .derive("kube::CustomResource");

    // Add kube::CustomResource attribute with additional parameters
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
                    let field_type = match &item.schema_kind {
                        openapiv3::SchemaKind::Type(openapiv3::Type::String(_)) => "String",
                        openapiv3::SchemaKind::Type(openapiv3::Type::Integer(_)) => "u32",
                        // Add more cases here for other types as needed
                        _ => continue, // Skip unknown types
                    };
                    struct_.field(field_name, field_type);
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
            handler: fn(WatchEvent<T>),
        ) -> anyhow::Result<()>
        where
            T: Clone + core::fmt::Debug + serde::de::DeserializeOwned + 'static,
        {
            let mut stream = api.watch(&watch_params, "0").await?.boxed();

            loop {
                while let Some(event) = stream.next().await {
                    match event {
                        Ok(event) => handler(event),
                        Err(e) => error!("Error watching resource: {:?}", e),
                    }
                }

                sleep(Duration::from_secs(1)).await;
                stream = api.watch(&watch_params, "0").await?.boxed();
            }
        }
    };

    // Convert the function into a string
    let function_string = quote! { #function }.to_string();

    // Add the function to the scope
    scope.raw(&function_string);
}

fn generate_function(name: &str) {
    let function_name = format_ident!("handle_{}", name.to_lowercase());
    let struct_name = format_ident!("{}", name);

    let function: ItemFn = parse_quote! {
        pub fn #function_name(event: WatchEvent<#struct_name>) {
            match event {
                WatchEvent::Added(resource) => {
                    info!("{} Added: {:?}", #name, resource.metadata.name);
                    todo!("TODO: Implement event handling");
                }
                WatchEvent::Modified(resource) => {
                    info!("{} Modified: {:?}", #name, resource.metadata.name);
                    todo!("TODO: Implement event handling");
                }
                WatchEvent::Deleted(resource) => {
                    info!("{} Deleted: {:?}", #name, resource.metadata.name);
                    todo!("TODO: Implement event handling");
                }
                _ => {}
            }
        }
    };

    // Convert the function into a string
    let function_string = quote! { #function }.to_string();

    // Prepend the function with the use statements
    let function_string = format!(
        "use kube::api::WatchEvent;\nuse log::info;use crate::{};\n\n{}",
        name, function_string
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
