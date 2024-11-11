use askama::Template;
use clap::Error;
use log::{debug, error, info};
use openapiv3::OpenAPI;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::{fs::DirBuilder, path::Path};

const K8S_TESTS_UTILS_DIR: &str = "tests/src/utils";

/// Creates a directory if it does not already exist.
///
/// # Arguments
///
/// * `dir` - A reference to the path of the directory to create.
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

/// Writes content to a file without filtering.
///
/// # Arguments
///
/// * `base_path` - The base path where the file is located.
/// * `file_name` - The name of the file to write to.
/// * `file_content` - The content to write to the file.
fn write_to_file_without_filter(base_path: &Path, file_name: &str, file_content: String) {
    let file_path = base_path.join(file_name);
    let file_path_str = file_path.to_string_lossy().to_string();

    match std::fs::write(&file_path, file_content + "\n") {
        Ok(_) => info!("Successfully wrote to file: {}", file_path_str),
        Err(e) => error!("Failed to write to file: {}. Error: {}", file_path_str, e),
    }
}

/// Generates a template file.
///
/// # Arguments
///
/// * `template` - The template to render.
/// * `base_path` - The base path where the file is located.
/// * `file_name` - The name of the file to write to.
pub fn generate_template_file<T: Template>(template: T, base_path: &Path, file_name: &str) {
    let content = template.render().unwrap();
    write_to_file_without_filter(base_path, file_name, content);
}

/// Sets executable permission for a file.
///
/// # Arguments
///
/// * `file_path` - The path of the file to set the permission for.
pub fn set_executable_permission(file_path: &Path) {
    let metadata = std::fs::metadata(file_path).expect("Unable to read file metadata");
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o755); // Set executable permission
    std::fs::set_permissions(file_path, permissions).expect("Unable to set file permissions");
}

/// Reads the OpenAPI specification from a file.
///
/// # Arguments
///
/// * `file_path` - The path of the OpenAPI specification file.
///
/// # Returns
///
/// This function returns an `OpenAPI` object.
pub fn read_openapi_spec(file_path: &str) -> OpenAPI {
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).expect("Unable to parse OpenAPI spec")
}

/// Creates a file if it does not already exist.
///
/// # Arguments
///
/// * `base_path` - The base path where the file is located.
/// * `file` - The name of the file to create.
pub fn create_file_if_not_exists(base_path: &Path, file: &str) {
    let file_path = base_path.join(file);
    if !file_path.exists() {
        File::create(&file_path)
            .unwrap_or_else(|_| panic!("Unable to create file {}", file_path.to_string_lossy()));
    }
}

/// Extracts information from the OpenAPI specification.
///
/// # Arguments
///
/// * `openapi` - A reference to the `OpenAPI` object.
///
/// # Returns
///
/// This function returns a tuple containing the extracted information.
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

/// Extracts a string extension from the OpenAPI specification.
///
/// # Arguments
///
/// * `openapi` - A reference to the `OpenAPI` object.
/// * `key` - The key of the extension to extract.
///
/// # Returns
///
/// This function returns the extracted string.
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

/// Extracts an array extension from the OpenAPI specification.
///
/// # Arguments
///
/// * `openapi` - A reference to the `OpenAPI` object.
/// * `key` - The key of the extension to extract.
///
/// # Returns
///
/// This function returns the extracted array as a vector of strings.
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

/// Adds a test utility module to the mod.rs file.
///
/// # Arguments
///
/// * `base_path` - The base path where the mod.rs file is located.
/// * `util_name` - The name of the utility module to add.
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

/// Upserts a line to a file without filtering.
///
/// # Arguments
///
/// * `file_path` - The path of the file to upsert the line to.
/// * `line` - The line to upsert.
///
/// # Returns
///
/// This function returns a `Result` indicating the success or failure of the operation.
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

/// Retrieves the list of ignored files.
///
/// # Returns
///
/// This function returns a vector of strings containing the paths of the ignored files.
pub fn get_ignored_files() -> Vec<String> {
    let ignore_file_path = ".openapi-generator-ignore";
    let ignore_file = File::open(ignore_file_path)
        .unwrap_or_else(|_| panic!("Unable to open file: {:?}", ignore_file_path));
    let reader = BufReader::new(ignore_file);
    reader.lines().map_while(Result::ok).collect()
}

/// Upserts a line to a file.
///
/// # Arguments
///
/// * `file_path` - The path of the file to upsert the line to.
/// * `line` - The line to upsert.
///
/// # Returns
///
/// This function returns a `Result` indicating the success or failure of the operation.
pub fn upsert_line_to_file(file_path: &str, line: &str) -> Result<(), Error> {
    if get_ignored_files().contains(&file_path.to_string()) {
        return Ok(());
    }

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

/// Writes content to a file.
///
/// # Arguments
///
/// * `base_path` - The base path where the file is located.
/// * `file_name` - The name of the file to write to.
/// * `file_content` - The content to write to the file.
pub fn write_to_file(base_path: &Path, file_name: &str, file_content: String) {
    let file_path = base_path.join(file_name);
    debug!("Writing to file: {}", file_path.to_string_lossy());
    if get_ignored_files().contains(&file_path.to_string_lossy().to_string()) {
        return;
    }

    std::fs::write(file_path, file_content + "\n").expect("Unable to write file");
}

/// Formats a file using rustfmt.
///
/// # Arguments
///
/// * `file_path` - The path of the file to format.
pub fn format_file(file_path: String) {
    if get_ignored_files().contains(&file_path) {
        return;
    }

    let output = Command::new("rustfmt")
        .arg(file_path)
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        error!(
            "rustfmt failed with output:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Uppercases the first letter of a string.
///
/// # Arguments
///
/// * `name` - The string to uppercase the first letter of.
///
/// # Returns
///
/// This function returns the modified string with the first letter uppercased.
pub fn uppercase_first_letter(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
