mod utils;

use kopgen::{
    commands::generate::{execute, generate_types},
    errors::AppError,
    utils::read_openapi_spec,
};
use openapiv3::{OpenAPI, Schema};
use std::{collections::HashMap, fs};
use tempfile::tempdir;
use utils::create_temp_file;

#[test]
fn test_execute_fails_because_of_missing_kubernetes_extension() -> Result<(), AppError> {
    let openapi_yaml = r#"---
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
paths: {}
components:
  schemas: {} 
"#;

    let (dir, openapi_file) = create_temp_file("openapi.yaml", openapi_yaml);

    let result = execute(
        &dir.path().to_string_lossy().to_string(),
        &openapi_file,
        &false,
        &false,
        &false,
        &false,
        &false,
    );

    assert!(
        result.is_err(),
        "Expected execution to fail due to missing Kubernetes extension."
    );

    match result {
        Err(AppError::MissingRequiredExtension(ref key)) => {
            assert_eq!(
                key, "x-kubernetes-operator-group",
                "Expected missing extension key to be 'x-kubernetes-operator-group'."
            );
        }
        Err(e) => {
            panic!("Expected MissingRequiredExtension error, but got: {:?}", e);
        }
        Ok(_) => {
            panic!(
                "Expected execution to fail due to missing Kubernetes extension, but it succeeded."
            );
        }
    }

    Ok(())
}

/// This test need to be fixed.

// #[test]
// fn test_execute_success() -> Result<(), AppError> {
//     let openapi_yaml = r#"---
// openapi: 3.0.0
// info:
//   title: Test API
//   version: 1.0.0
//   x-kubernetes-operator-group: testgroup
//   x-kubernetes-operator-version: v1
//   x-kubernetes-operator-resource-ref: uuid
//   x-kubernetes-operator-status-ref: status
//   x-kubernetes-operator-include-tags:
//     - test
//     - example
//   x-kubernetes-operator-example-metadata-spec-field-ref: metadata
// paths: {}
// components:
//   schemas: {}
// "#;
//     let (_dir, openapi_file) = create_temp_file("openapi.yaml", openapi_yaml);

//     let all = true;
//     let lib = false;
//     let manifests = false;
//     let controllers = false;
//     let types = false;

//     // TODO - pass to this function a base directory to write the generated files and use relative paths
//     let result = execute(&openapi_file, &all, &lib, &manifests, &controllers, &types)?;

//     // assert!(result.is_ok());

//     Ok(())
// }

#[test]
fn test_execute_missing_openapi_file() -> Result<(), AppError> {
    let dir = tempdir().unwrap();
    let openapi_file = dir.path().join("missing_openapi.yaml");

    let all = true;
    let lib = false;
    let manifests = false;
    let controllers = false;
    let types = false;

    let result = execute(
        &dir.path().to_string_lossy().to_string(),
        &openapi_file.to_string_lossy().to_string(),
        &all,
        &lib,
        &manifests,
        &controllers,
        &types,
    );

    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_execute_invalid_openapi_spec() -> Result<(), AppError> {
    let openapi_yaml = r#"---
invalid_yaml: [unclosed_list
"#;
    let (dir, openapi_file) = create_temp_file("invalid_openapi.yaml", openapi_yaml);

    let all = true;
    let lib = false;
    let manifests = false;
    let controllers = false;
    let types = false;

    let result = execute(
        &dir.path().to_string_lossy().to_string(),
        &openapi_file,
        &all,
        &lib,
        &manifests,
        &controllers,
        &types,
    );

    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_generate_types() -> Result<(), AppError> {
    let openapi_yaml = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
  x-kubernetes-operator-group: testgroup
  x-kubernetes-operator-version: v1
  x-kubernetes-operator-resource-ref: uuid
  x-kubernetes-operator-status-ref: status
  x-kubernetes-operator-include-tags:
    - user
  x-kubernetes-operator-example-metadata-spec-field-ref: metadata
paths: {}
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: integer
          format: int64
        name:
          type: string
      required:
        - id
"#;

    let (dir, openapi_file_path) = create_temp_file("openapi.yaml", openapi_yaml);
    let output_path: std::path::PathBuf = dir.path().join("src").join("types");
    fs::create_dir_all(&output_path).unwrap();

    let openapi: OpenAPI = read_openapi_spec(openapi_file_path.as_str())?;

    let schemas: HashMap<String, Schema> = openapi
        .components
        .ok_or_else(|| AppError::Other("No components found in OpenAPI spec".to_string()))?
        .schemas
        .iter()
        .filter_map(|(name, schema)| match schema {
            openapiv3::ReferenceOr::Item(schema) => Some((name.clone(), schema.clone())),
            openapiv3::ReferenceOr::Reference { .. } => None,
        })
        .collect();

    let operator_resource_ref = "id".to_string();
    let types_directory = output_path
        .to_str()
        .expect("Failed to convert output path to string");

    let result = generate_types(types_directory, schemas, &operator_resource_ref);

    assert!(result.is_ok());

    let type_file = output_path.join("user.rs");

    assert!(type_file.exists(), "Type file was not created");

    let generated_content = fs::read_to_string(&type_file)?;
    assert!(
        generated_content.contains("struct User"),
        "Generated content does not contain expected struct"
    );

    dir.close()?;
    Ok(())
}
