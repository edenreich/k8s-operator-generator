use dotenv::dotenv;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub kubernetes_operator_group: String,
    pub kubernetes_operator_version: String,
    pub kubernetes_operator_resource_ref: String,
    pub kubernetes_operator_example_metadata_spec_field_ref: String,
    pub kubernetes_operator_include_tags: Vec<String>,
}

impl Config {
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

pub trait ConfigProvider {
    fn load_config() -> Result<Config, String>;
}

pub struct EnvConfigProvider;

impl ConfigProvider for EnvConfigProvider {
    fn load_config() -> Result<Config, String> {
        Config::from_env()
    }
}
