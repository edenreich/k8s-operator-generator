use crate::utils::{read_openapi_spec_raw, write_openapi_spec_raw};
use log::info;
use serde_yaml::Value as YamlValue;
use std::process::Command as ProcessCommand;

pub fn execute(openapi_file: &String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Hydrating OpenAPI spec...");
    let mut openapi = read_openapi_spec_raw(openapi_file);

    if let Some(info) = openapi.get_mut("info") {
        if let Some(info_map) = info.as_mapping_mut() {
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-group".to_string()),
                YamlValue::String(std::env::var("KUBERNETES_OPERATOR_GROUP").unwrap_or_default()),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-version".to_string()),
                YamlValue::String(std::env::var("KUBERNETES_OPERATOR_VERSION").unwrap_or_default()),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-resource-ref".to_string()),
                YamlValue::String(
                    std::env::var("KUBERNETES_OPERATOR_RESOURCE_REF").unwrap_or_default(),
                ),
            );
            let kubernetes_operator_include_tags =
                std::env::var("KUBERNETES_OPERATOR_INCLUDE_TAGS")
                    .expect("KUBERNETES_OPERATOR_INCLUDE_TAGS environment variable not set");
            let tags_list: Vec<YamlValue> = kubernetes_operator_include_tags
                .split(',')
                .map(|tag| YamlValue::String(tag.trim().to_string()))
                .collect();
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-include-tags".to_string()),
                YamlValue::Sequence(tags_list),
            );
            info_map.insert(
                YamlValue::String(
                    "x-kubernetes-operator-example-metadata-spec-field-ref".to_string(),
                ),
                YamlValue::String(
                    std::env::var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF")
                        .unwrap_or_default(),
                ),
            );
        }
    }

    write_openapi_spec_raw(openapi_file, &openapi);

    let _ = ProcessCommand::new("prettier")
        .arg("--write")
        .arg(openapi_file)
        .output()
        .expect("Failed to run prettier on OpenAPI spec");

    info!("OpenAPI spec hydrated successfully");
    Ok(())
}
