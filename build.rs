use codegen::Scope;
use openapiv3::{OpenAPI, Schema};
use serde_yaml;
use std::fs::File;
use std::io::Read;

fn main() {
    let input = "./openapi.yaml";
    let output = "./src/lib.rs";

    // Read the OpenAPI specification from the YAML file
    let mut file = File::open(input).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    // Parse the OpenAPI specification
    let openapi: OpenAPI = serde_yaml::from_str(&contents).expect("Unable to parse OpenAPI spec");

    // Create a new codegen scope
    let mut scope = Scope::new();

    // Generate Rust structs for each schema in the components
    if let Some(components) = openapi.components {
        for (name, schema) in components.schemas {
            match schema {
                openapiv3::ReferenceOr::Reference { .. } => {
                    // Handle references here if needed
                }
                openapiv3::ReferenceOr::Item(item) => {
                    generate_struct(&mut scope, &name, "example.com", "v1", &item);
                }
            }
        }
    }
    // Write the generated code to a file
    std::fs::write(output, format!("{}\n", scope.to_string())).expect("Unable to write file");
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
