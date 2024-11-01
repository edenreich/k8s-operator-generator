mod utils;

use std::env;

use kopgen::{
    commands,
    config::{ConfigProvider, EnvConfigProvider},
};
use serde_yaml::{Mapping, Value as YamlValue};
use serial_test::serial;
use utils::{create_temp_file, read_temp_file};

fn setup_env() {
    env::set_var(
        "TARGET_ARCH",
        format!("{}-unknown-linux-musl", std::env::consts::ARCH),
    );
    env::set_var(
        "OPENAPI_DOWNLOAD_URL",
        "https://github.com/edenreich/kopgen/blob/main/openapi.yaml",
    );
    env::set_var("INSTALL_CRDS", "true");
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("CONTAINER_REGISTRY", "localhost:5005");
    env::set_var("CLUSTER_NAME", "k3d-k3s-default");
    env::set_var("RELEASE", "false");
    env::set_var("KUBERNETES_OPERATOR_GROUP", "test-group");
    env::set_var("KUBERNETES_OPERATOR_VERSION", "v1");
    env::set_var("KUBERNETES_OPERATOR_RESOURCE_REF", "test-resource-ref");
    env::set_var("KUBERNETES_OPERATOR_INCLUDE_TAGS", "tag1,tag2");
    env::set_var(
        "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
        "test-field-ref",
    );
}

fn clear_env() {
    env::remove_var("OPENAPI_DOWNLOAD_URL");
    env::remove_var("INSTALL_CRDS");
    env::remove_var("RUST_LOG");
    env::remove_var("RUST_BACKTRACE");
    env::remove_var("CONTAINER_REGISTRY");
    env::remove_var("CLUSTER_NAME");
    env::remove_var("TARGET_ARCH");
    env::remove_var("RELEASE");
    env::remove_var("KUBERNETES_OPERATOR_GROUP");
    env::remove_var("KUBERNETES_OPERATOR_VERSION");
    env::remove_var("KUBERNETES_OPERATOR_RESOURCE_REF");
    env::remove_var("KUBERNETES_OPERATOR_INCLUDE_TAGS");
    env::remove_var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF");
}

#[test]
#[serial]
fn test_execute_hydrates_openapi_spec() {
    setup_env();

    let (dir, file_path) = create_temp_file(
        "openapi.yaml",
        r#"
info:
  title: Test API
  version: 1.0.0
"#,
    );

    let conf = EnvConfigProvider::load_config().expect("Failed to load configuration");

    commands::hydrate::execute(&file_path, &conf).unwrap();

    let yaml: YamlValue = read_temp_file(&file_path).expect("Unable to read file");
    let info: &Mapping = yaml.get("info").unwrap().as_mapping().unwrap();

    assert_eq!(
        info.get(YamlValue::String("x-kubernetes-operator-group".to_string()))
            .unwrap()
            .as_str()
            .unwrap(),
        "test-group"
    );
    assert_eq!(
        info.get(YamlValue::String(
            "x-kubernetes-operator-version".to_string()
        ))
        .unwrap()
        .as_str()
        .unwrap(),
        "v1"
    );
    assert_eq!(
        info.get(YamlValue::String(
            "x-kubernetes-operator-resource-ref".to_string()
        ))
        .unwrap()
        .as_str()
        .unwrap(),
        "test-resource-ref"
    );
    assert_eq!(
        info.get(YamlValue::String(
            "x-kubernetes-operator-include-tags".to_string()
        ))
        .unwrap()
        .as_sequence()
        .unwrap(),
        &vec![
            YamlValue::String("tag1".to_string()),
            YamlValue::String("tag2".to_string())
        ]
    );
    assert_eq!(
        info.get(YamlValue::String(
            "x-kubernetes-operator-example-metadata-spec-field-ref".to_string()
        ))
        .unwrap()
        .as_str()
        .unwrap(),
        "test-field-ref"
    );

    drop(dir);
    clear_env();
}
