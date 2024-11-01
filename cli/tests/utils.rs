use serde_yaml::Error;
use serde_yaml::Value as YamlValue;
use std::io::Write;
use std::{env, fs::File};
use tempfile::TempDir;

pub fn setup_test_env() {
    env::set_var("RUST_LOG", "info");

    env::set_var("KUBERNETES_OPERATOR_GROUP", "test-group");
    env::set_var("KUBERNETES_OPERATOR_VERSION", "v1");
    env::set_var("KUBERNETES_OPERATOR_RESOURCE_REF", "test-resource-ref");
    env::set_var("KUBERNETES_OPERATOR_INCLUDE_TAGS", "tag1,tag2");
    env::set_var(
        "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
        "test-field-ref",
    );
}

// This function is used to create a temporary file with the provided content
#[allow(dead_code)]
pub fn create_temp_file(file_name: &str, content: &str) -> (TempDir, String) {
    let dir: TempDir = TempDir::new().unwrap();
    let file_path = dir.path().join(file_name);
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "{}", content).unwrap();
    (dir, file_path.to_string_lossy().to_string())
}

// This function is used to read the content of a temporary file
#[allow(dead_code)]
pub fn read_temp_file(file_path: &str) -> Result<YamlValue, Error> {
    let hydrated_spec_content = std::fs::read_to_string(file_path).expect("Unable to read file");
    serde_yaml::from_str(&hydrated_spec_content)
}
