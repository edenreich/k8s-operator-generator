#[cfg(test)]
/// Tests for the [`kopgen::commands::init`](cli/src/commands/init_command.rs) of the `kopgen` CLI.
mod tests {
    use assert_cmd::Command;
    use kopgen::errors::AppError;
    use predicates::prelude::*;
    use serial_test::serial;
    use std::{env, fs};
    use tempfile::tempdir;

    /// Sets up the necessary environment variables for testing.
    fn setup_env() {
        env::set_var("RUST_LOG", "info");
        env::set_var("KUBERNETES_OPERATOR_NAME", "Custom Operator Name");
        env::set_var("KUBERNETES_OPERATOR_AUTHOR", "Custom Author");
        env::set_var("KUBERNETES_OPERATOR_API_GROUP", "test-group");
        env::set_var("KUBERNETES_OPERATOR_API_VERSION", "v1");
        env::set_var("KUBERNETES_OPERATOR_RESOURCE_REF", "test-resource-ref");
        env::set_var("KUBERNETES_OPERATOR_INCLUDE_TAGS", "tag1,tag2");
        env::set_var(
            "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
            "test-field-ref",
        );
    }

    /// Clears the environment variables set for testing.
    fn clear_env() {
        env::remove_var("RUST_LOG");
        env::remove_var("KUBERNETES_OPERATOR_NAME");
        env::remove_var("KUBERNETES_OPERATOR_AUTHOR");
        env::remove_var("KUBERNETES_OPERATOR_API_GROUP");
        env::remove_var("KUBERNETES_OPERATOR_API_VERSION");
        env::remove_var("KUBERNETES_OPERATOR_RESOURCE_REF");
        env::remove_var("KUBERNETES_OPERATOR_INCLUDE_TAGS");
        env::remove_var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF");
    }

    /// Runs the `init` command and asserts success.
    fn run_init_command(path: &str) {
        Command::cargo_bin("kopgen")
            .unwrap()
            .arg("init")
            .arg(path)
            .assert()
            .success()
            .stderr(predicate::str::contains("Initialized project at"));
    }

    /// Checks if the specified directories exist.
    fn assert_directories_exist(path: &std::path::Path, dirs: &[&str]) {
        for dir in dirs {
            assert!(path.join(dir).exists(), "Directory {} was not created", dir);
        }
    }

    /// Checks if the specified files exist.
    fn assert_files_exist(path: &std::path::Path, files: &[&str]) {
        for file in files {
            let file_path = path.join(file);
            assert!(
                file_path.exists(),
                "File {} was not created",
                file_path.display()
            );
        }
    }

    /// Tests that the `init` command creates all required directories.
    #[test]
    #[serial]
    fn init_creates_required_directories() -> Result<(), AppError> {
        setup_env();

        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        run_init_command(path.to_str().unwrap());

        let expected_dirs = [
            ".cargo",
            ".devcontainer",
            "operator",
            "crdgen",
            "crdgen/src",
            "tests",
            "tests/src/utils",
            "operator/src/controllers",
            "operator/src/types",
            "manifests/crds",
            "manifests/rbac",
            "manifests/operator",
            "manifests/examples",
        ];

        assert_directories_exist(path, &expected_dirs);
        Ok(())
    }

    /// Tests that the `init` command creates all required files.
    #[test]
    #[serial]
    fn init_creates_required_files() -> Result<(), AppError> {
        setup_env();

        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        run_init_command(path.to_str().unwrap());

        let expected_files = [
            ".openapi-generator-ignore",
            "Cargo.toml",
            ".dockerignore",
            ".editorconfig",
            ".env.example",
            ".gitattributes",
            ".gitignore",
            "Taskfile.yml",
            "README.md",
            ".prettierrc.yaml",
            ".rustfmt.toml",
            "Cluster.yaml",
            "Dockerfile",
            "operator/Cargo.toml",
            "crdgen/Cargo.toml",
            "tests/Cargo.toml",
            ".devcontainer/devcontainer.json",
            ".devcontainer/Dockerfile",
            ".devcontainer/setup-git.sh",
            ".devcontainer/launch.json",
            ".devcontainer/.zshrc",
            ".cargo/config.toml",
            "operator/src/main.rs",
            "operator/src/cli.rs",
            "operator/src/controllers/mod.rs",
            "operator/src/types/mod.rs",
            "tests/src/utils/mod.rs",
            "tests/src/utils/client.rs",
            "tests/src/utils/operator.rs",
            "tests/src/utils/cluster.rs",
            "tests/main.rs",
        ];

        assert_files_exist(path, &expected_files);

        clear_env();
        Ok(())
    }

    /// Tests that the `init` command skips initialization if the directory already exists and is not empty.
    #[test]
    #[serial]
    fn init_skips_existing_directory() -> Result<(), AppError> {
        setup_env();

        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        // Pre-create the directory to simulate an existing path
        fs::create_dir_all(path).unwrap();

        // Place a file in the directory to ensure it's not removed
        fs::write(path.join("existing-file.txt"), "Hello, world!").unwrap();

        // Run the `init` command
        Command::cargo_bin("kopgen")
            .unwrap()
            .arg("init")
            .arg(path)
            .assert()
            .success()
            .stderr(predicate::str::contains(
                "Directory already exists and is not empty",
            ));

        // Verify no other files are created
        assert_eq!(fs::read_dir(path).unwrap().count(), 1);

        clear_env();
        Ok(())
    }
}
