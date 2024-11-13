use crate::config::Config;
use crate::errors::AppError;
use log::info;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::process::Command as ProcessCommand;

/// Executes the hydration process for the OpenAPI specification.
///
/// This function updates the OpenAPI specification with additional metadata
/// from the provided configuration.
///
/// # Arguments
///
/// * `openapi_file` - A string slice that holds the path to the OpenAPI file.
/// * `config` - A reference to the configuration object containing metadata to be added.
///
/// # Returns
///
/// This function returns a `Result` indicating the success or failure of the operation.
pub fn execute(openapi_file: &String, config: &Config) -> Result<(), AppError> {
    info!("Hydrating OpenAPI spec...");

    let mut openapi: YamlValue = serde_yaml::from_str(&fs::read_to_string(openapi_file)?)?;

    let tags_list: Vec<YamlValue> = config
        .kubernetes_operator_include_tags
        .iter()
        .map(|tag| YamlValue::String(tag.to_string()))
        .collect();

    if let Some(info) = openapi.get_mut("info") {
        if let Some(info_map) = info.as_mapping_mut() {
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-group".to_string()),
                YamlValue::String(config.kubernetes_operator_group.clone()),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-version".to_string()),
                YamlValue::String(config.kubernetes_operator_version.clone()),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-resource-ref".to_string()),
                YamlValue::String(config.kubernetes_operator_resource_ref.clone()),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-include-tags".to_string()),
                YamlValue::Sequence(tags_list),
            );
            info_map.insert(
                YamlValue::String(
                    "x-kubernetes-operator-example-metadata-spec-field-ref".to_string(),
                ),
                YamlValue::String(
                    config
                        .kubernetes_operator_example_metadata_spec_field_ref
                        .clone(),
                ),
            );
        }
    }

    fs::write(openapi_file, serde_yaml::to_string(&openapi)?)?;

    let _ = ProcessCommand::new("npx")
        .arg("prettier")
        .arg("--write")
        .arg(openapi_file)
        .output()
        .expect("Failed to run prettier on OpenAPI spec");

    info!("OpenAPI spec hydrated successfully");

    Ok(())
}
