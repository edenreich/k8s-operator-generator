use kopgen::errors::AppError;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

/// This function is used to create a temporary file with the provided content
pub fn create_temp_file(file_name: &str, content: &str) -> (TempDir, String) {
    let dir: TempDir = TempDir::new().unwrap();
    let file_path = dir.path().join(file_name);
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "{}", content).unwrap();
    (dir, file_path.to_string_lossy().to_string())
}

pub enum SpecValue {
    Yaml(YamlValue),
    Json(JsonValue),
}

/// This function is used to read the content of a temporary file
pub fn read_temp_file(file_path: &str) -> Result<SpecValue, AppError> {
    let hydrated_spec_content = std::fs::read_to_string(file_path)?;

    if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
        let yaml = serde_yaml::from_str(&hydrated_spec_content)?;
        Ok(SpecValue::Yaml(yaml))
    } else if file_path.ends_with(".json") {
        let json = serde_json::from_str(&hydrated_spec_content)?;
        Ok(SpecValue::Json(json))
    } else {
        // Attempt to parse as YAML, fallback to JSON
        match serde_yaml::from_str(&hydrated_spec_content) {
            Ok(yaml) => Ok(SpecValue::Yaml(yaml)),
            Err(_) => {
                let json = serde_json::from_str(&hydrated_spec_content)?;
                Ok(SpecValue::Json(json))
            }
        }
    }
}
