#[cfg(test)]
/// Tests for the [`kopgen::commands::generate`](cli/src/commands/generate_command.rs) of the `kopgen` CLI.
mod tests {
    use crate::utils::create_temp_file;
    use kopgen::{
        commands::generate::{execute, generate_types},
        errors::AppError,
        utils::read_openapi_spec,
    };
    use openapiv3::Schema;
    use serial_test::serial;
    use std::{collections::HashMap, fs};
    use tempfile::tempdir;

    /// Helper function to set common parameters for `execute`.
    fn default_params() -> (bool, bool, bool, bool) {
        (true, false, false, false)
    }

    /// Tests that `execute` fails when the Kubernetes extension is missing from the OpenAPI spec.
    #[test]
    #[serial]
    fn test_execute_fails_missing_kubernetes_extension() -> Result<(), AppError> {
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
        let (all, manifests, controllers, types) = default_params();

        let result = execute(
            &dir.path().to_string_lossy().to_string(),
            &openapi_file,
            &all,
            &manifests,
            &controllers,
            &types,
        );

        assert!(
            result.is_err(),
            "Expected execution to fail due to missing Kubernetes extension."
        );

        if let Err(AppError::MissingRequiredExtension(ref key)) = result {
            assert_eq!(
                key, "x-kubernetes-operator-api-group",
                "Expected missing extension key to be 'x-kubernetes-operator-api-group'."
            );
        } else {
            panic!("Expected MissingRequiredExtension error.");
        }

        Ok(())
    }

    /// Tests that `execute` returns an error when the OpenAPI file is missing.
    #[test]
    #[serial]
    fn test_execute_missing_openapi_file() -> Result<(), AppError> {
        let dir = tempdir()?;
        let openapi_file = dir.path().join("missing_openapi.yaml");

        let (all, manifests, controllers, types) = default_params();

        let result = execute(
            &dir.path().to_string_lossy().to_string(),
            &openapi_file.to_string_lossy().to_string(),
            &all,
            &manifests,
            &controllers,
            &types,
        );

        assert!(
            result.is_err(),
            "Expected error due to missing OpenAPI file."
        );

        Ok(())
    }

    /// Tests that `execute` returns an error when the OpenAPI spec is invalid.
    #[test]
    #[serial]
    fn test_execute_invalid_openapi_spec() -> Result<(), AppError> {
        let openapi_yaml = r#"---
invalid_yaml: [unclosed_list
"#;
        let (dir, openapi_file) = create_temp_file("invalid_openapi.yaml", openapi_yaml);
        let (all, manifests, controllers, types) = default_params();

        let result = execute(
            &dir.path().to_string_lossy().to_string(),
            &openapi_file,
            &all,
            &manifests,
            &controllers,
            &types,
        );

        assert!(
            result.is_err(),
            "Expected execution to fail due to invalid OpenAPI spec."
        );

        Ok(())
    }

    /// Tests that `generate_types` successfully generates type files from a valid OpenAPI spec.
    #[test]
    #[serial]
    fn test_generate_types_success() -> Result<(), AppError> {
        let openapi_yaml = r#"
openapi: 3.0.0
info:
  title: Test API
  version: 1.0.0
  x-kubernetes-operator-api-group: testgroup
  x-kubernetes-operator-api-version: v1
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
        let output_path = dir.path().join("src").join("types");
        fs::create_dir_all(&output_path)?;

        let openapi = read_openapi_spec(openapi_file_path.as_str())?;
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

        generate_types(types_directory, &schemas, &operator_resource_ref)?;

        let type_file = output_path.join("user.rs");
        assert!(type_file.exists(), "Type file was not created.");

        let generated_content = fs::read_to_string(&type_file)?;
        assert!(
            generated_content.contains("struct User"),
            "Generated content does not contain expected struct."
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_parsing_json_spec() -> Result<(), AppError> {
        let openapi_json = r#"{
    "openapi": "3.0.0",
    "info": {
        "title": "Test API",
        "version": "1.0.0",
        "x-kubernetes-operator-api-group": "testgroup",
        "x-kubernetes-operator-api-version": "v1",
        "x-kubernetes-operator-resource-ref": "uuid",
        "x-kubernetes-operator-status-ref": "status",
        "x-kubernetes-operator-include-tags": ["user"],
        "x-kubernetes-operator-example-metadata-spec-field-ref": "metadata"
    },
    "paths": {},
    "components": {
        "schemas": {
            "User": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "format": "int64"
                    },
                    "name": {
                        "type": "string"
                    },
                    "email": {
                        "type": "string",
                        "format": "email"
                    }
                },
                "required": ["id"]
            }
        }
    }
}"#;

        let (dir, openapi_file) = create_temp_file("openapi.json", openapi_json);
        let output_path = dir.path().join("src").join("types");
        fs::create_dir_all(&output_path)?;

        let openapi = read_openapi_spec(openapi_file.as_str())?;
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

        generate_types(types_directory, &schemas, &operator_resource_ref)?;

        let type_file = output_path.join("user.rs");
        assert!(type_file.exists(), "Type file was not created.");

        let generated_content = fs::read_to_string(&type_file)?;
        assert!(
            generated_content.contains("struct User"),
            "Generated content does not contain expected User struct."
        );
        assert!(
            generated_content.contains("struct UserSpec"),
            "Generated content does not contain expected UserSpec struct."
        );
        assert!(
            generated_content.contains("struct UserStatus"),
            "Generated content does not contain expected UserStatus struct."
        );
        assert!(
            generated_content.contains("pub name: Option<String>"),
            "Generated content does not contain expected name field."
        );
        assert!(
            generated_content.contains("pub email: Option<String>"),
            "Generated content does not contain expected email field."
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_invalid_json_spec() -> Result<(), AppError> {
        let openapi_json = r#"{
    "openapi": "3.0.0",
    "info": {
        "title": "Test API",
        "version": "1.0.0",
        "x-kubernetes-operator-api-group": "testgroup",
        "x-kubernetes-operator-api-version": "v1",
        "x-kubernetes-operator-resource-ref": "uuid",
        "x-kubernetes-operator-status-ref": "status",
        "x-kubernetes-operator-include-tags": ["user"],
        "x-kubernetes-operator-example-metadata-spec-field-ref": "metadata"
    },
    "paths": {},
    "components": {
        "schemas": {
            "User": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "format": "int64"
                    },
                    "name": {
                        "type": "string"
                    },
                    "email": {
                        "type": "string",
                        "format": "email"
                    }
                },
                "required": ["id"]
            }
        }
    }
"#; // Missing closing brace error

        let (dir, openapi_file) = create_temp_file("invalid_openapi.json", openapi_json);
        let output_path = dir.path().join("src").join("types");
        fs::create_dir_all(&output_path)?;

        let openapi = read_openapi_spec(openapi_file.as_str());
        assert!(openapi.is_err(), "Expected error due to invalid JSON spec.");
        if let Err(e) = openapi {
            match e {
                AppError::JsonError(_) => (),
                _ => panic!("Expected AppError::JsonError, got {:?}", e),
            }
        }

        Ok(())
    }
}
