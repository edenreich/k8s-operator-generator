#[cfg(test)]
/// Tests for the [`kopgen::config::Config`](cli/src/config.rs) of the `kopgen` CLI.
mod tests {
    use clap::Parser;
    use kopgen::cli::Cli;
    use kopgen::config::{Config, ConfigProvider};
    use kopgen::errors::AppError;
    use serial_test::serial;
    use std::env; // Ensure that Cli derives Parser

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

    /// Clears all environment variables set during testing.
    fn clear_env() {
        env::remove_var("KUBERNETES_OPERATOR_NAME");
        env::remove_var("KUBERNETES_OPERATOR_AUTHOR");
        env::remove_var("KUBERNETES_OPERATOR_API_GROUP");
        env::remove_var("KUBERNETES_OPERATOR_API_VERSION");
        env::remove_var("KUBERNETES_OPERATOR_RESOURCE_REF");
        env::remove_var("KUBERNETES_OPERATOR_INCLUDE_TAGS");
        env::remove_var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF");
    }

    /// Tests that the configuration is correctly loaded from environment variables.
    #[test]
    #[serial]
    fn test_config_from_env() -> Result<(), AppError> {
        setup_env();

        let config = Config::load_from_env()?;

        assert_eq!(config.operator_name, "Custom Operator Name");
        assert_eq!(config.operator_author, "Custom Author");
        assert_eq!(config.api_group, "test-group");
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.resource_ref, "test-resource-ref");
        assert_eq!(config.example_metadata_spec_field_ref, "test-field-ref");
        assert_eq!(config.include_tags, vec!["tag1", "tag2"]);

        clear_env();
        Ok(())
    }

    /// Tests that comma-separated tags are correctly parsed into a vector.
    #[test]
    #[serial]
    fn test_config_parses_comma_separated_tags_as_vector() -> Result<(), AppError> {
        setup_env();

        let config = Config::load_from_env()?;

        assert_eq!(config.include_tags, vec!["tag1", "tag2"]);

        clear_env();
        Ok(())
    }

    /// Tests that missing optional environment variables results in default values being used.
    #[test]
    #[serial]
    fn test_config_missing_optional_env_vars() -> Result<(), AppError> {
        setup_env();

        // Remove optional environment variables
        env::remove_var("KUBERNETES_OPERATOR_NAME");
        env::remove_var("KUBERNETES_OPERATOR_AUTHOR");

        let config = Config::load_from_env()?;

        // Verify that default values are used for optional fields
        assert_eq!(config.operator_name, "Example Operator");
        assert_eq!(config.operator_author, "Unknown");

        // Verify that required fields are still loaded correctly
        assert_eq!(config.api_group, "test-group");
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.resource_ref, "test-resource-ref");
        assert_eq!(config.example_metadata_spec_field_ref, "test-field-ref");
        assert_eq!(config.include_tags, vec!["tag1", "tag2"]);

        clear_env();
        Ok(())
    }

    /// Tests that setting environment variables overrides the default values.
    #[test]
    #[serial]
    fn test_config_env_overrides_defaults() -> Result<(), AppError> {
        setup_env();

        // Override optional environment variables
        env::set_var("KUBERNETES_OPERATOR_NAME", "Overridden Operator Name");
        env::set_var("KUBERNETES_OPERATOR_AUTHOR", "Overridden Author");

        let config = Config::load_from_env()?;

        // Verify that overridden values are used
        assert_eq!(config.operator_name, "Overridden Operator Name");
        assert_eq!(config.operator_author, "Overridden Author");

        clear_env();
        Ok(())
    }

    /// Tests that the configuration is correctly loaded from CLI arguments.
    #[test]
    #[serial]
    fn test_config_load_from_cli_with_args() -> Result<(), AppError> {
        // Clear environment variables to ensure CLI args are used exclusively
        clear_env();

        // Simulate CLI arguments
        let cli_args = vec![
            "kopgen",
            "--kubernetes-operator-name",
            "CLI Operator",
            "--kubernetes-operator-author",
            "CLI Author",
            "--kubernetes-operator-api-group",
            "cli-group",
            "--kubernetes-operator-api-version",
            "v2",
            "--kubernetes-operator-resource-ref",
            "cli-resource-ref",
            "--kubernetes-operator-include-tags",
            "cli-tag1,cli-tag2",
            "--kubernetes-operator-example-metadata-spec-field-ref",
            "cli-field-ref",
        ];

        // Parse CLI arguments into a Cli instance
        let cli = Cli::parse_from(cli_args);

        let config = Config::load_from_cli(&cli)?;

        // Verify that CLI arguments are correctly loaded
        assert_eq!(config.operator_name, "CLI Operator");
        assert_eq!(config.operator_author, "CLI Author");
        assert_eq!(config.api_group, "cli-group");
        assert_eq!(config.api_version, "v2");
        assert_eq!(config.resource_ref, "cli-resource-ref");
        assert_eq!(config.example_metadata_spec_field_ref, "cli-field-ref");
        assert_eq!(config.include_tags, vec!["cli-tag1", "cli-tag2"]);

        Ok(())
    }

    /// Tests that CLI arguments override environment variables.
    #[test]
    #[serial]
    fn test_config_load_from_cli_overrides_env_vars() -> Result<(), AppError> {
        setup_env();

        // Simulate CLI arguments that override some environment variables
        let cli_args = vec![
            "kopgen",
            "--kubernetes-operator-name",
            "CLI Operator Override",
            "--kubernetes-operator-api-group",
            "cli-group-override",
        ];

        // Parse CLI arguments into a Cli instance
        let cli = Cli::parse_from(cli_args);

        // Load configuration from CLI arguments
        let config = Config::load_from_cli(&cli)?;

        // Verify that CLI arguments override environment variables
        assert_eq!(config.operator_name, "CLI Operator Override");
        assert_eq!(config.operator_author, "Custom Author"); // From env
        assert_eq!(config.api_group, "cli-group-override");
        assert_eq!(config.api_version, "v1"); // From env
        assert_eq!(config.resource_ref, "test-resource-ref"); // From env
        assert_eq!(config.example_metadata_spec_field_ref, "test-field-ref"); // From env
        assert_eq!(config.include_tags, vec!["tag1", "tag2"]); // From env

        clear_env();
        Ok(())
    }

    /// Tests that the configuration defaults are used when neither environment variables nor CLI arguments are provided.
    #[test]
    #[serial]
    fn test_config_load_from_cli_without_args() -> Result<(), AppError> {
        // Ensure no environment variables are set
        clear_env();

        // Simulate no CLI arguments except the program name
        let cli_args = vec!["kopgen", "--kubernetes-operator-name", "Example Operator"];

        // Parse CLI arguments into a Cli instance
        let cli = Cli::parse_from(cli_args);

        // Load configuration from CLI arguments
        let config = Config::load_from_cli(&cli)?;

        // Verify that default values are used
        assert_eq!(config.operator_name, "Example Operator");
        assert_eq!(config.operator_author, "Unknown");
        assert_eq!(config.api_group, "example.com");
        assert_eq!(config.api_version, "v1");
        assert_eq!(config.resource_ref, "uuid");
        assert_eq!(config.example_metadata_spec_field_ref, "name");
        assert_eq!(config.include_tags, Vec::<String>::new());

        Ok(())
    }
}
