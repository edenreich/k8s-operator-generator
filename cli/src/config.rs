use dotenvy::dotenv;
use std::env;

use crate::{cli::Cli, errors::AppError};

/// Configuration for the Kubernetes Operator Generator tool.
///
/// This struct holds the configuration values required for the tool, which are
/// typically loaded from environment variables or CLI arguments.
#[derive(Debug, Clone)]
pub struct Config {
    pub operator_name: String,
    pub operator_author: String,
    pub api_group: String,
    pub api_version: String,
    pub resource_ref: String,
    pub example_metadata_spec_field_ref: String,
    pub include_tags: Vec<String>,
}

impl Config {
    /// Loads the configuration from environment variables.
    ///
    /// This function reads the necessary environment variables and constructs
    /// a `Config` object. If any required environment variable is not set,
    /// it returns an error.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the `Config` object or an `AppError`.
    pub fn from_env() -> Result<Self, AppError> {
        dotenv().ok();

        Ok(Self {
            operator_name: Self::get_env_var_or_default(
                "KUBERNETES_OPERATOR_NAME",
                "Example Operator",
            ),
            operator_author: Self::get_env_var_or_default("KUBERNETES_OPERATOR_AUTHOR", "Unknown"),
            api_group: Self::get_env_var_or_default("KUBERNETES_OPERATOR_API_GROUP", "example.com"),
            api_version: Self::get_env_var_or_default("KUBERNETES_OPERATOR_API_VERSION", "v1"),
            resource_ref: Self::get_env_var_or_default("KUBERNETES_OPERATOR_RESOURCE_REF", "uuid"),
            example_metadata_spec_field_ref: Self::get_env_var_or_default(
                "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
                "name",
            ),
            include_tags: Self::get_env_var_or_default("KUBERNETES_OPERATOR_INCLUDE_TAGS", "")
                .split(',')
                .filter(|tag| !tag.is_empty())
                .map(|tag| tag.trim().to_string())
                .collect(),
        })
    }

    /// Creates a `Config` object from CLI arguments.
    ///
    /// This function takes parsed CLI arguments and constructs a `Config` object.
    ///
    /// # Arguments
    ///
    /// * `cli` - A reference to the `Cli` struct containing parsed CLI arguments.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the `Config` object or an `AppError`.
    pub fn from_cli(cli: &Cli) -> Result<Self, AppError> {
        Ok(Self {
            operator_name: cli.kubernetes_operator_name.clone(),
            operator_author: cli.kubernetes_operator_author.clone(),
            api_group: cli.kubernetes_operator_api_group.clone(),
            api_version: cli.kubernetes_operator_api_version.clone(),
            resource_ref: cli.kubernetes_operator_resource_ref.clone(),
            example_metadata_spec_field_ref: cli
                .kubernetes_operator_example_metadata_spec_field_ref
                .clone(),
            include_tags: cli.kubernetes_operator_include_tags.clone(),
        })
    }

    /// Helper function to retrieve environment variables with error handling.
    ///
    /// # Arguments
    ///
    /// * `key` - The environment variable key to retrieve.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the environment variable's value
    /// or an `AppError` if the variable is not set.
    #[allow(dead_code)]
    fn get_env_var(key: &str) -> Result<String, AppError> {
        env::var(key)
            .map_err(|_| AppError::ConfigError(format!("{} environment variable not set", key)))
    }

    /// Helper function to retrieve environment variables or default.
    ///
    /// # Arguments
    ///
    /// * `key` - The environment variable key to retrieve.
    /// * `default` - The default value.
    ///
    /// # Returns
    ///
    /// This function returns a `String` containing the environment variable's value
    fn get_env_var_or_default(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }
}

/// Trait for loading configuration.
///
/// This trait defines methods for loading the configuration from different sources.
pub trait ConfigProvider {
    /// Loads the configuration from environment variables.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the `Config` object or an `AppError`.
    #[allow(unused_variables, dead_code)]
    fn load_from_env() -> Result<Config, AppError> {
        Config::from_env()
    }

    /// Loads the configuration from CLI arguments.
    ///
    /// # Arguments
    ///
    /// * `cli` - A reference to the `Cli` struct containing parsed CLI arguments.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the `Config` object or an `AppError`.
    fn load_from_cli(cli: &Cli) -> Result<Config, AppError> {
        Config::from_cli(cli)
    }
}

// Implement the ConfigProvider trait for Config
impl ConfigProvider for Config {}
