use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn init_creates_required_directories() {
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
fn init_creates_required_files() {
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
}

#[test]
fn init_skips_existing_directory() {
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
}
