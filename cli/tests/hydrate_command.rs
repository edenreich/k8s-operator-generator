mod utils;

use kopgen::commands;
use serde_yaml::{Mapping, Value as YamlValue};
use serial_test::serial;
use std::env;
use utils::{create_temp_file, read_temp_file, setup_test_env};

#[test]
#[serial]
fn test_execute_hydrates_openapi_spec() {
    setup_test_env();

    let (dir, file_path) = create_temp_file(
        "openapi.yaml",
        r#"
info:
  title: Test API
  version: 1.0.0
"#,
    );

    commands::hydrate::execute(&file_path).unwrap();

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
}

#[test]
#[serial]
#[should_panic(expected = "KUBERNETES_OPERATOR_INCLUDE_TAGS environment variable not set")]
fn test_execute_missing_include_tags_env_var() {
    setup_test_env();

    env::remove_var("KUBERNETES_OPERATOR_INCLUDE_TAGS");

    let (dir, file_path) = create_temp_file(
        "openapi.yaml",
        r#"
info:
  title: Test API
  version: 1.0.0
"#,
    );

    commands::hydrate::execute(&file_path).unwrap();

    drop(dir);
}
