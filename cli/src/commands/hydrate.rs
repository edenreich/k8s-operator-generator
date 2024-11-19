use crate::{config::Config, errors::AppError};
use log::info;
use serde_yaml::Value as YamlValue;
use std::fs;

/// Executes the hydration process for the OpenAPI specification.
///
/// This function updates the OpenAPI specification with additional metadata
/// from the provided configuration.
///
/// # Arguments
///
/// * `conf` - A reference to the configuration object containing metadata to be added.
/// * `openapi_file` - A string slice that holds the path to the OpenAPI file.
///
/// # Returns
///
/// This function returns a `Result` indicating the success or failure of the operation.
pub fn execute(conf: Config, openapi_file: &String) -> Result<(), AppError> {
    info!("Hydrating OpenAPI spec...");

    let mut openapi: YamlValue = serde_yaml::from_str(&fs::read_to_string(openapi_file)?)?;

    openapi = match openapi {
        YamlValue::Null => {
            return Err(AppError::ConfigError(
                "OpenAPI spec file is empty".to_string(),
            ));
        }
        _ => openapi,
    };

    if let Some(openapi_map) = openapi.as_mapping() {
        if openapi_map
            .get(YamlValue::String("openapi".to_string()))
            .is_none()
        {
            return Err(AppError::ConfigError(
                "OpenAPI spec is missing required 'openapi' field".to_string(),
            ));
        }
        if openapi_map
            .get(YamlValue::String("info".to_string()))
            .is_none()
        {
            return Err(AppError::ConfigError(
                "OpenAPI spec is missing required 'info' field".to_string(),
            ));
        }
    }

    let tags_list: Vec<YamlValue> = conf
        .include_tags
        .iter()
        .map(|tag| YamlValue::String(tag.to_string()))
        .collect();

    if let Some(info) = openapi.get_mut("info") {
        if let Some(info_map) = info.as_mapping_mut() {
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-api-group".to_string()),
                YamlValue::String(conf.api_group),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-api-version".to_string()),
                YamlValue::String(conf.api_version),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-resource-ref".to_string()),
                YamlValue::String(conf.resource_ref),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-include-tags".to_string()),
                YamlValue::Sequence(tags_list),
            );
            info_map.insert(
                YamlValue::String(
                    "x-kubernetes-operator-example-metadata-spec-field-ref".to_string(),
                ),
                YamlValue::String(conf.example_metadata_spec_field_ref),
            );
        }
    }

    fs::write(openapi_file, serde_yaml::to_string(&openapi)?)?;

    info!("OpenAPI spec hydrated successfully");

    Ok(())
}
