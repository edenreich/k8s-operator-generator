#[cfg(test)]
/// Tests for the `init` command of the `kopgen` CLI.
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use serial_test::serial;
    use std::{env, fs};
    use tempfile::tempdir;

    /// Sets up the necessary environment variables for testing.
    fn setup_env() {
        env::set_var("KUBERNETES_OPERATOR_NAME", "Custom Operator Name");
        env::set_var("KUBERNETES_OPERATOR_AUTHOR", "Custom Author");
        env::set_var("KUBERNETES_OPERATOR_GROUP", "test-group");
        env::set_var("KUBERNETES_OPERATOR_VERSION", "v1");
        env::set_var("KUBERNETES_OPERATOR_RESOURCE_REF", "test-resource-ref");
        env::set_var("KUBERNETES_OPERATOR_INCLUDE_TAGS", "tag1,tag2");
        env::set_var(
            "KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF",
            "test-field-ref",
        );
    }

    /// Clears the environment variables set for testing.
    fn clear_env() {
        env::remove_var("KUBERNETES_OPERATOR_NAME");
        env::remove_var("KUBERNETES_OPERATOR_AUTHOR");
        env::remove_var("KUBERNETES_OPERATOR_GROUP");
        env::remove_var("KUBERNETES_OPERATOR_VERSION");
        env::remove_var("KUBERNETES_OPERATOR_RESOURCE_REF");
        env::remove_var("KUBERNETES_OPERATOR_INCLUDE_TAGS");
        env::remove_var("KUBERNETES_OPERATOR_EXAMPLE_METADATA_SPEC_FIELD_REF");
    }

    /// Tests that the `init` command creates all required directories.
    #[test]
    #[serial]
    fn init_creates_required_directories() {
        setup_env();

        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        // Run the `init` command using the CLI
        Command::cargo_bin("kopgen")
            .unwrap()
            .arg("init")
            .arg(path.to_str().unwrap())
            .assert()
            .success()
            .stderr(predicate::str::contains("Initialized project at")); // TODO - investigate why the env_logger by default all goes to stderr, it's weird.

        // Check the expected directories exist
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

        for dir in &expected_dirs {
            assert!(path.join(dir).exists(), "Directory {} was not created", dir);
        }
    }

    /// Tests that the `init` command creates all required files.
    #[test]
    #[serial]
    fn init_creates_required_files() {
        setup_env();

        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path();

        // Run the `init` command
        Command::cargo_bin("kopgen")
            .unwrap()
            .arg("init")
            .arg(path)
            .assert()
            .success();

        // Check the expected files exist
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
            ".devcontainer/deps.sh",
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

        for file in expected_files {
            let file_path = path.join(file);
            assert!(
                file_path.exists(),
                "File {} was not created",
                file_path.display()
            );
        }

        clear_env();
    }

    /// Tests that the `init` command skips initialization if the directory already exists and is not empty.
    #[test]
    #[serial]
    fn init_skips_existing_directory() {
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
    }
}
