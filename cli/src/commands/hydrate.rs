use crate::config::Config;
use log::info;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::process::Command as ProcessCommand;

pub fn execute(openapi_file: &String, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
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
