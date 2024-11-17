#[cfg(test)]
mod tests {
    use kopgen::config::{Config, ConfigProvider};
    use serial_test::serial;
    use std::env;

    /// Sets up the necessary environment variables for testing.
    fn setup_env() {
        env::set_var(
            "TARGET_ARCH",
            format!("{}-unknown-linux-musl", std::env::consts::ARCH),
        );
        env::set_var(
            "OPENAPI_DOWNLOAD_URL",
            "https://github.com/edenreich/kopgen/blob/main/openapi.yaml",
        );
        env::set_var("INSTALL_CRDS", "true");
        env::set_var("RUST_LOG", "info");
        env::set_var("RUST_BACKTRACE", "1");
        env::set_var("CONTAINER_REGISTRY", "localhost:5005");
        env::set_var("CLUSTER_NAME", "k3d-k3s-default");
        env::set_var("RELEASE", "false");
        env::set_var("KUBERNETES_OPERATOR_API_GROUP", "test-group");
        env::set_var("KUBERNETES_OPERATOR_API_VERSION", "v1");
        env::set_var("KUBERNETES_OPERATOR_RESOURCE_REF", "test-resource-ref");
        env::set_var("KUBERNETES_OPERATOR_INCLUDE_TAGS", "tag1,tag2");
        env::set_var(
            "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
            "test-field-ref",
        );
        // Optional environment variables with default values
        env::set_var("KUBERNETES_OPERATOR_NAME", "Custom Operator Name");
        env::set_var("KUBERNETES_OPERATOR_AUTHOR", "Custom Author");
    }

    /// Clears all environment variables set during testing.
    fn clear_env() {
        env::remove_var("TARGET_ARCH");
        env::remove_var("OPENAPI_DOWNLOAD_URL");
        env::remove_var("INSTALL_CRDS");
        env::remove_var("RUST_LOG");
        env::remove_var("RUST_BACKTRACE");
        env::remove_var("CONTAINER_REGISTRY");
        env::remove_var("CLUSTER_NAME");
        env::remove_var("RELEASE");
        env::remove_var("KUBERNETES_OPERATOR_API_GROUP");
        env::remove_var("KUBERNETES_OPERATOR_API_VERSION");
        env::remove_var("KUBERNETES_OPERATOR_RESOURCE_REF");
        env::remove_var("KUBERNETES_OPERATOR_INCLUDE_TAGS");
        env::remove_var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF");
        env::remove_var("KUBERNETES_OPERATOR_NAME");
        env::remove_var("KUBERNETES_OPERATOR_AUTHOR");
    }

    /// Tests that the configuration is correctly loaded from environment variables.
    #[test]
    #[serial]
    fn test_config_from_env() {
        setup_env();

        let config = Config::load_from_env().expect("Failed to load configuration");

        assert_eq!(config.api_group, "test-group");
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.resource_ref, "test-resource-ref");
        assert_eq!(config.example_metadata_spec_field_ref, "test-field-ref");
        assert_eq!(config.include_tags, vec!["tag1", "tag2"]);
        assert_eq!(config.operator_name, "Custom Operator Name");
        assert_eq!(config.operator_author, "Custom Author");

        clear_env();
    }

    /// Tests that comma-separated tags are correctly parsed into a vector.
    #[test]
    #[serial]
    fn test_config_parses_comma_separated_tags_as_vector() {
        setup_env();

        let config = Config::load_from_env().expect("Failed to load configuration");

        assert_eq!(config.include_tags, vec!["tag1", "tag2"]);

        clear_env();
    }

    /// Tests that all required environment variables are present and configuration loads successfully.
    #[test]
    #[serial]
    fn test_all_required_env_vars_present() {
        setup_env();

        let result = Config::load_from_env();

        assert!(
            result.is_ok(),
            "Configuration should load successfully when all required environment variables are set"
        );

        let config = result.unwrap();

        // Verify that environment variables override default values
        assert_eq!(config.operator_name, "Custom Operator Name");
        assert_eq!(config.operator_author, "Custom Author");

        assert_eq!(config.api_group, "test-group");
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.resource_ref, "test-resource-ref");
        assert_eq!(config.example_metadata_spec_field_ref, "test-field-ref");
        assert_eq!(config.include_tags, vec!["tag1", "tag2"]);

        clear_env();
    }

    /// Tests that missing optional environment variables results in default values being used.
    #[test]
    #[serial]
    fn test_config_missing_optional_env_vars() {
        setup_env();

        // Remove optional environment variables
        env::remove_var("KUBERNETES_OPERATOR_NAME");
        env::remove_var("KUBERNETES_OPERATOR_AUTHOR");

        let result = Config::load_from_env().expect("Failed to load configuration");

        // Verify that default values are used for optional fields
        assert_eq!(result.operator_name, "Example Operator");
        assert_eq!(result.operator_author, "Example");

        // Verify that required fields are still loaded correctly
        assert_eq!(result.api_group, "test-group");
        assert_eq!(result.api_version, "v1");
        assert_eq!(result.resource_ref, "test-resource-ref");
        assert_eq!(result.example_metadata_spec_field_ref, "test-field-ref");
        assert_eq!(result.include_tags, vec!["tag1", "tag2"]);

        clear_env();
    }

    /// Tests that setting environment variables overrides the default values.
    #[test]
    #[serial]
    fn test_config_env_overrides_defaults() {
        setup_env();

        // Override optional environment variables
        env::set_var("KUBERNETES_OPERATOR_NAME", "Overridden Operator Name");
        env::set_var("KUBERNETES_OPERATOR_AUTHOR", "Overridden Author");

        let config = Config::load_from_env().expect("Failed to load configuration");

        // Verify that overridden values are used
        assert_eq!(config.operator_name, "Overridden Operator Name");
        assert_eq!(config.operator_author, "Overridden Author");

        clear_env();
    }
}
