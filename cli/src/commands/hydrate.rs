use log::error;
use log::info;
use serde_yaml::Value as YamlValue;
use std::env;
use std::fs;
use std::process::Command as ProcessCommand;

pub fn execute(openapi_file: &String) -> Result<(), Box<dyn std::error::Error>> {
    info!("Hydrating OpenAPI spec...");

    let mut openapi: YamlValue = serde_yaml::from_str(&fs::read_to_string(openapi_file)?)?;

    let kubernetes_operator_group = env::var("KUBERNETES_OPERATOR_GROUP")
        .map_err(|e| {
            error!(
                "KUBERNETES_OPERATOR_GROUP environment variable not set: {}",
                e
            );
            e
        })
        .expect("KUBERNETES_OPERATOR_GROUP environment variable not set");

    let kubernetes_operator_version = env::var("KUBERNETES_OPERATOR_VERSION")
        .map_err(|e| {
            error!(
                "KUBERNETES_OPERATOR_VERSION environment variable not set: {}",
                e
            );
            e
        })
        .expect("KUBERNETES_OPERATOR_VERSION environment variable not set");

    let kubernetes_operator_resource_ref = env::var("KUBERNETES_OPERATOR_RESOURCE_REF")
        .map_err(|e| {
            error!(
                "KUBERNETES_OPERATOR_RESOURCE_REF environment variable not set: {}",
                e
            );
            e
        })
        .expect("KUBERNETES_OPERATOR_RESOURCE_REF environment variable not set");

    let kubernetes_operator_include_tags = env::var("KUBERNETES_OPERATOR_INCLUDE_TAGS")
        .map_err(|e| {
            error!(
                "KUBERNETES_OPERATOR_INCLUDE_TAGS environment variable not set: {}",
                e
            );
            e
        })
        .expect("KUBERNETES_OPERATOR_INCLUDE_TAGS environment variable not set");

    let tags_list: Vec<YamlValue> = kubernetes_operator_include_tags
        .split(',')
        .map(|tag| YamlValue::String(tag.trim().to_string()))
        .collect();

    let kubernetes_operator_example_metadata_spec_field_ref =
        env::var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF")
            .map_err(|e| {
                error!(
            "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF environment variable not set: {}",
            e
        );
                e
            })
            .expect(
                "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF environment variable not set",
            );

    if let Some(info) = openapi.get_mut("info") {
        if let Some(info_map) = info.as_mapping_mut() {
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-group".to_string()),
                YamlValue::String(kubernetes_operator_group),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-version".to_string()),
                YamlValue::String(kubernetes_operator_version),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-resource-ref".to_string()),
                YamlValue::String(kubernetes_operator_resource_ref),
            );
            info_map.insert(
                YamlValue::String("x-kubernetes-operator-include-tags".to_string()),
                YamlValue::Sequence(tags_list),
            );
            info_map.insert(
                YamlValue::String(
                    "x-kubernetes-operator-example-metadata-spec-field-ref".to_string(),
                ),
                YamlValue::String(kubernetes_operator_example_metadata_spec_field_ref),
            );
        }
    }

    fs::write(openapi_file, serde_yaml::to_string(&openapi)?)?;

    let _ = ProcessCommand::new("prettier")
        .arg("--write")
        .arg(openapi_file)
        .output()
        .expect("Failed to run prettier on OpenAPI spec");

    info!("OpenAPI spec hydrated successfully");

    Ok(())
}
