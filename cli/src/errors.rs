use askama::Error as AskamaError;
use serde_json::Error as JsonError;
use serde_yaml::Error as YamlError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Missing required extension: {0}")]
    MissingRequiredExtension(String),

    #[error("YAML deserialization error: {0}")]
    YamlError(#[from] YamlError),

    #[error("JSON deserialization error: {0}")]
    JsonError(#[from] JsonError),

    #[error("Askama template error: {0}")]
    AskamaError(#[from] AskamaError),

    #[error("An IO error occurred")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}
