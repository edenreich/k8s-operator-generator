use dotenv::dotenv;
use std::env;

/// Configuration for the Kubernetes Operator Generator tool.
///
/// This struct holds the configuration values required for the tool, which are
/// typically loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    pub kubernetes_operator_group: String,
    pub kubernetes_operator_version: String,
    pub kubernetes_operator_resource_ref: String,
    pub kubernetes_operator_example_metadata_spec_field_ref: String,
    pub kubernetes_operator_include_tags: Vec<String>,
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
    /// This function returns a `Result` containing the `Config` object or an error message.
    pub fn from_env() -> Result<Self, String> {
        dotenv().ok();

        Ok(Config {
            kubernetes_operator_group: env::var("KUBERNETES_OPERATOR_GROUP")
                .map_err(|_| "KUBERNETES_OPERATOR_GROUP environment variable not set")?,
            kubernetes_operator_version: env::var("KUBERNETES_OPERATOR_VERSION")
                .map_err(|_| "KUBERNETES_OPERATOR_VERSION environment variable not set")?,
            kubernetes_operator_resource_ref: env::var("KUBERNETES_OPERATOR_RESOURCE_REF")
                .map_err(|_| "KUBERNETES_OPERATOR_RESOURCE_REF environment variable not set")?,
            kubernetes_operator_example_metadata_spec_field_ref: env::var(
                "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
            )
            .map_err(|_| {
                "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF environment variable not set"
            })?,
            kubernetes_operator_include_tags: env::var("KUBERNETES_OPERATOR_INCLUDE_TAGS")
                .map_err(|_| "KUBERNETES_OPERATOR_INCLUDE_TAGS environment variable not set")?
                .split(',')
                .map(|tag| tag.trim().to_string())
                .collect(),
        })
    }
}

/// Trait for loading configuration.
///
/// This trait defines a method for loading the configuration, which can be
/// implemented by different configuration providers.
pub trait ConfigProvider {
    /// Loads the configuration.
    ///
    /// This function should be implemented to load the configuration from a specific source.
    ///
    /// # Returns
    ///
    /// This function returns a `Result` containing the `Config` object or an error message.
    fn load_config() -> Result<Config, String>;
}

/// Environment variable-based configuration provider.
///
/// This struct implements the `ConfigProvider` trait to load the configuration
/// from environment variables.
pub struct EnvConfigProvider;

impl ConfigProvider for EnvConfigProvider {
    fn load_config() -> Result<Config, String> {
        Config::from_env()
    }
}
