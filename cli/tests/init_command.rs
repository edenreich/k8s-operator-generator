mod utils;

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::{env, fs};
use tempfile::tempdir;

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
fn init_creates_required_directories() {
    setup_env();

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    // Run the `init` command using the CLI
    Command::cargo_bin("kopgen")
        .unwrap()
        .arg("init")
        .arg(path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized project at"));

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
        ".prettierrc",
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
