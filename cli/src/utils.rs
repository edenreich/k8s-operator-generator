use askama::Template;
use clap::Error;
use indexmap::IndexMap;
use log::{error, info};
use openapiv3::OpenAPI;
use serde_yaml::Value as YamlValue;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::{fs::DirBuilder, path::Path};

const K8S_TESTS_UTILS_DIR: &str = "tests/src/utils";

pub fn create_directory_if_not_exists(dir: &Path) {
    let path_str = dir.to_string_lossy().to_string();
    if dir.exists() {
        info!("Directory {} already exists. skipping...", path_str);
    } else {
        DirBuilder::new()
            .recursive(true)
            .create(dir)
            .unwrap_or_else(|_| panic!("Unable to create {} directory", path_str));
    }
}

fn write_to_file_without_filter(base_path: &Path, file_name: &str, file_content: String) {
    let file_path = base_path.join(file_name);
    let file_path_str = file_path.to_string_lossy().to_string();

    match std::fs::write(&file_path, file_content + "\n") {
        Ok(_) => info!("Successfully wrote to file: {}", file_path_str),
        Err(e) => error!("Failed to write to file: {}. Error: {}", file_path_str, e),
    }
}

pub fn generate_template_file<T: Template>(template: T, base_path: &Path, file_name: &str) {
    let content = template.render().unwrap();
    write_to_file_without_filter(base_path, file_name, content);
}

pub fn set_executable_permission(file_path: &Path) {
    let metadata = std::fs::metadata(file_path).expect("Unable to read file metadata");
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755); // Set executable permission
    std::fs::set_permissions(file_path, permissions).expect("Unable to set file permissions");
}

pub fn read_openapi_spec_raw(file_path: &str) -> IndexMap<String, YamlValue> {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Unable to parse OpenAPI spec")
}

pub fn read_openapi_spec(file_path: &str) -> OpenAPI {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Unable to parse OpenAPI spec")
}

pub fn write_openapi_spec_raw(file_path: &str, openapi: &IndexMap<String, YamlValue>) {
    let file = File::create(file_path).expect("Unable to create file");
    serde_yaml::to_writer(file, openapi).expect("Unable to write OpenAPI spec");
}

pub fn create_file_if_not_exists(base_path: &Path, file: &str) {
    let file_path = base_path.join(file);
    if !file_path.exists() {
        File::create(&file_path)
            .unwrap_or_else(|_| panic!("Unable to create file {}", file_path.to_string_lossy()));
    }
}

pub fn extract_openapi_info(openapi: &OpenAPI) -> (String, String, String, Vec<String>, String) {
    let kubernetes_operator_group = extract_extension(openapi, "x-kubernetes-operator-group");
    let kubernetes_operator_version = extract_extension(openapi, "x-kubernetes-operator-version");
    let kubernetes_operator_resource_ref =
        extract_extension(openapi, "x-kubernetes-operator-resource-ref");
    let kubernetes_operator_include_tags =
        extract_extension_array(openapi, "x-kubernetes-operator-include-tags");
    let kubernetes_operator_metadata_spec_field_name = extract_extension(
        openapi,
        "x-kubernetes-operator-example-metadata-spec-field-ref",
    );
    (
        kubernetes_operator_group,
        kubernetes_operator_version,
        kubernetes_operator_resource_ref,
        kubernetes_operator_include_tags,
        kubernetes_operator_metadata_spec_field_name,
    )
}

fn extract_extension(openapi: &OpenAPI, key: &str) -> String {
    openapi
        .info
        .extensions
        .get(key)
        .unwrap_or_else(|| panic!("No {} in OpenAPI spec", key))
        .as_str()
        .unwrap_or_else(|| panic!("{} is not a string", key))
        .to_string()
}

fn extract_extension_array(openapi: &OpenAPI, key: &str) -> Vec<String> {
    openapi
        .info
        .extensions
        .get(key)
        .unwrap_or_else(|| panic!("No {} in OpenAPI spec", key))
        .as_array()
        .unwrap_or_else(|| panic!("{} is not an array", key))
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect()
}

pub fn add_tests_util_to_modfile(base_path: &Path, util_name: &str) {
    let file_path = base_path.join(K8S_TESTS_UTILS_DIR).join("mod.rs");
    let file_path_str = file_path.to_string_lossy().to_string();
    match upsert_line_to_file_without_filter(
        &file_path_str,
        format!("pub mod {};", util_name.to_lowercase()).as_str(),
    ) {
        Ok(_) => (),
        Err(e) => error!(
            "Failed to add utility module '{}' to '{}'. Error: {}",
            util_name.to_lowercase(),
            file_path_str,
            e
        ),
    }
}

fn upsert_line_to_file_without_filter(file_path: &str, line: &str) -> Result<(), Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let exists = reader.lines().any(|l| l.unwrap() == line);

    if !exists {
        let mut file = OpenOptions::new().append(true).open(file_path)?;
        if let Err(e) = writeln!(file, "{}", line) {
            error!("Couldn't write to file: {}", e);
        }
    }
    Ok(())
}
