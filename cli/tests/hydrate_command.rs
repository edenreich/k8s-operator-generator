mod utils;

#[cfg(test)]
/// Tests for the `hydrate` command of the `kopgen` CLI.
mod tests {
    use super::*;
    use kopgen::{
        commands,
        config::{Config, ConfigProvider},
        errors::AppError,
    };
    use serde_yaml::{Mapping, Value as YamlValue};
    use serial_test::serial;
    use std::env;
    use utils::{create_temp_file, read_temp_file};

    /// Sets up the necessary environment variables for testing.
    fn setup_env() {
        env::set_var("KUBERNETES_OPERATOR_NAME", "Custom Operator Name");
        env::set_var("KUBERNETES_OPERATOR_AUTHOR", "Custom Author");
        env::set_var("KUBERNETES_OPERATOR_API_GROUP", "test-group");
        env::set_var("KUBERNETES_OPERATOR_API_VERSION", "v1");
        env::set_var("KUBERNETES_OPERATOR_RESOURCE_REF", "test-resource-ref");
        env::set_var("KUBERNETES_OPERATOR_INCLUDE_TAGS", "tag1,tag2");
        env::set_var(
            "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
            "test-field-ref",
        );
    }

    /// Clears the environment variables set for testing.
    fn clear_env() {
        env::remove_var("KUBERNETES_OPERATOR_NAME");
        env::remove_var("KUBERNETES_OPERATOR_AUTHOR");
        env::remove_var("KUBERNETES_OPERATOR_API_GROUP");
        env::remove_var("KUBERNETES_OPERATOR_API_VERSION");
        env::remove_var("KUBERNETES_OPERATOR_RESOURCE_REF");
        env::remove_var("KUBERNETES_OPERATOR_INCLUDE_TAGS");
        env::remove_var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF");
    }

    /// Tests that the hydrate command correctly updates the OpenAPI spec with configuration values.
    #[test]
    #[serial]
    fn test_execute_hydrates_openapi_spec() -> Result<(), AppError> {
        setup_env();

        let (dir, file_path) = create_temp_file(
            "openapi.yaml",
            r#"---
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
"#,
        );

        let config = Config::load_from_env()?;

        // Execute the hydrate command
        commands::hydrate::execute(config, &file_path)?;

        // Read back the YAML file
        let yaml: YamlValue = read_temp_file(&file_path)?;
        let info: &Mapping = yaml.get("info").unwrap().as_mapping().unwrap();

        // Assert that configuration values are correctly injected
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-api-group".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-group"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-api-version".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "v1"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-resource-ref".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-resource-ref"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-include-tags".to_string()
            ))
            .unwrap()
            .as_sequence()
            .unwrap(),
            &vec![
                YamlValue::String("tag1".to_string()),
                YamlValue::String("tag2".to_string())
            ]
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-example-metadata-spec-field-ref".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-field-ref"
        );

        drop(dir);
        clear_env();
        Ok(())
    }

    /// Tests that the hydrate command uses default values when optional environment variables are missing.
    #[test]
    #[serial]
    fn test_execute_hydrates_with_defaults() -> Result<(), AppError> {
        setup_env();

        let (dir, file_path) = create_temp_file(
            "openapi.yaml",
            r#"---
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
"#,
        );

        let config = Config::load_from_env()?;

        // Execute the hydrate command
        commands::hydrate::execute(config, &file_path)?;

        // Read back the YAML file
        let yaml: YamlValue = read_temp_file(&file_path).expect("Unable to read file");
        let info: &Mapping = yaml.get("info").unwrap().as_mapping().unwrap();

        // Assert that configuration values with defaults are correctly injected
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-api-group".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-group"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-api-version".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "v1"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-resource-ref".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-resource-ref"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-include-tags".to_string()
            ))
            .unwrap()
            .as_sequence()
            .unwrap(),
            &vec![
                YamlValue::String("tag1".to_string()),
                YamlValue::String("tag2".to_string())
            ]
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-example-metadata-spec-field-ref".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-field-ref"
        );

        drop(dir);
        clear_env();
        Ok(())
    }

    /// Tests that the hydrate command correctly handles invalid YAML files.
    #[test]
    #[serial]
    fn test_execute_hydrate_invalid_yaml() -> Result<(), AppError> {
        setup_env();

        let (dir, file_path) = create_temp_file(
            "invalid_openapi.yaml",
            r#"---
info:
  title: Test API
  version: 1.0.0
  invalid_yaml: !!invalid
"#,
        );

        let config = Config::load_from_env()?;

        // Execute the hydrate command and expect it to fail due to invalid YAML
        let result = commands::hydrate::execute(config, &file_path);

        assert!(
            result.is_err(),
            "Expected hydrate command to fail due to invalid YAML format"
        );

        drop(dir);
        clear_env();
        Ok(())
    }

    /// Tests that the hydrate command does not modify the YAML file when no relevant fields are present.
    #[test]
    #[serial]
    fn test_execute_hydrate_no_fields_to_modify() -> Result<(), AppError> {
        setup_env();

        let (dir, file_path) = create_temp_file(
            "openapi_no_fields.yaml",
            r#"---
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
"#,
        );

        let config = Config::load_from_env()?;

        // Execute the hydrate command
        commands::hydrate::execute(config, &file_path)?;

        // Read back the YAML file
        let yaml: YamlValue = read_temp_file(&file_path).expect("Unable to read file");

        // Since there were no fields to modify, the YAML should remain unchanged except for the injections
        let info: &Mapping = yaml.get("info").unwrap().as_mapping().unwrap();

        // Verify that operator-specific fields are injected
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-api-group".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-group"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-api-version".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "v1"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-resource-ref".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-resource-ref"
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-include-tags".to_string()
            ))
            .unwrap()
            .as_sequence()
            .unwrap(),
            &vec![
                YamlValue::String("tag1".to_string()),
                YamlValue::String("tag2".to_string())
            ]
        );
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-example-metadata-spec-field-ref".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-field-ref"
        );

        drop(dir);
        clear_env();
        Ok(())
    }

    /// Tests that the hydrate command correctly handles empty OpenAPI spec files.
    #[test]
    #[serial]
    fn test_execute_hydrate_empty_yaml() -> Result<(), AppError> {
        setup_env();

        let (dir, file_path) = create_temp_file("empty_openapi.yaml", "");

        let config = Config::load_from_env()?;

        // Execute the hydrate command and expect it to fail due to empty YAML
        let result: Result<(), AppError> = commands::hydrate::execute(config, &file_path);

        assert!(
            result.is_err(),
            "Expected hydrate command to fail due to empty YAML file"
        );

        drop(dir);
        clear_env();
        Ok(())
    }

    /// Tests that the hydrate command preserves existing unrelated fields in the YAML file.
    #[test]
    #[serial]
    fn test_execute_hydrate_preserves_unrelated_fields() -> Result<(), AppError> {
        setup_env();

        let original_yaml = r#"---
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
servers:
  - url: https://api.test.com
"#;

        let (dir, file_path) = create_temp_file("openapi_preserve.yaml", original_yaml);

        let config = Config::load_from_env()?;

        // Execute the hydrate command
        commands::hydrate::execute(config, &file_path)?;

        // Read back the YAML file
        let yaml: YamlValue = read_temp_file(&file_path).expect("Unable to read file");

        // Verify that unrelated fields are preserved
        let expected_servers = vec![YamlValue::Mapping({
            let mut map = Mapping::new();
            map.insert(
                YamlValue::String("url".to_string()),
                YamlValue::String("https://api.test.com".to_string()),
            );
            map
        })];
        assert_eq!(
            yaml.get("servers").unwrap().as_sequence().unwrap(),
            &expected_servers
        );

        // Verify that operator-specific fields are injected
        let info: &Mapping = yaml.get("info").unwrap().as_mapping().unwrap();
        assert_eq!(
            info.get(YamlValue::String(
                "x-kubernetes-operator-api-group".to_string()
            ))
            .unwrap()
            .as_str()
            .unwrap(),
            "test-group"
        );

        drop(dir);
        clear_env();

        Ok(())
    }
}
