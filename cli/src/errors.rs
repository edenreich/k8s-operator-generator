use askama::Error as AskamaError;
use serde_yaml::Error as YamlError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Failed to load environment variables")]
    ConfigError(#[from] std::env::VarError),

    #[error("Missing required extension: {0}")]
    MissingRequiredExtension(String),

    #[error("YAML deserialization error: {0}")]
    YamlError(#[from] YamlError),

    #[error("Askama template error: {0}")]
    AskamaError(#[from] AskamaError),

    #[error("An IO error occurred")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}
