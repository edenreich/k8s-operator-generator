use kopgen::errors::AppError;
use serde_yaml::Value as YamlValue;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

// This function is used to create a temporary file with the provided content
pub fn create_temp_file(file_name: &str, content: &str) -> (TempDir, String) {
    let dir: TempDir = TempDir::new().unwrap();
    let file_path = dir.path().join(file_name);
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "{}", content).unwrap();
    (dir, file_path.to_string_lossy().to_string())
}

// This function is used to read the content of a temporary file
#[allow(dead_code)]
pub fn read_temp_file(file_path: &str) -> Result<YamlValue, AppError> {
    let hydrated_spec_content = std::fs::read_to_string(file_path).expect("Unable to read file");
    Ok(serde_yaml::from_str(&hydrated_spec_content)?)
}
