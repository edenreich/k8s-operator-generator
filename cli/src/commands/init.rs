use crate::errors::AppError;
use crate::templates::general::{
    CargoConfig, ClusterYaml, Dockerfile, Dockerignore, Editorconfig, EnvExample, GitAttributes,
    GitIgnore, Prettierrc, ReadmeMd, RustfmtToml, Taskfile,
};
use crate::templates::operator::Cli;
use crate::templates::{
    cargo::{CargoToml, CrdgenCargoToml, OperatorCargoToml, TestsCargoToml},
    crdgen::Main as CrdgenMain,
    devcontainer::{Deps, Json, LaunchJsonExample, SetupGit, Zshrc},
    general::OpenAPIGeneratorIgnore,
    operator::Main as OperatorMain,
    tests::{Main as TestsMain, UtilsClient, UtilsCluster, UtilsOperator},
};
use crate::utils::{
    add_tests_util_to_modfile, create_directory_if_not_exists, create_file_if_not_exists,
    generate_template_file, set_executable_permission,
};
use log::{info, warn};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

const CARGO_DIR: &str = ".cargo";
const DEVCONTAINER_DIR: &str = ".devcontainer";
const K8S_OPERATOR_DIR: &str = "operator";
const K8S_CRDGEN_DIR: &str = "crdgen";
const K8S_TESTS_DIR: &str = "tests";
const K8S_TESTS_UTILS_DIR: &str = "tests/src/utils";
const K8S_OPERATOR_CONTROLLERS_DIR: &str = "operator/src/controllers";
const K8S_OPERATOR_TYPES_DIR: &str = "operator/src/types";
const K8S_MANIFESTS_CRDS_DIR: &str = "manifests/crds";
const K8S_MANIFESTS_RBAC_DIR: &str = "manifests/rbac";
const K8S_MANIFESTS_OPERATOR_DIR: &str = "manifests/operator";
const K8S_MANIFESTS_EXAMPLES_DIR: &str = "manifests/examples";

/// Executes the initialization process for the project directory structure.
///
/// This function sets up the necessary directory structure and generates
/// template files for the project based on the provided path.
///
/// # Arguments
///
/// * `path` - A string slice that holds the path to the project directory.
///
/// # Returns
///
/// This function returns a `Result` indicating the success or failure of the operation.
pub fn execute(path: &String) -> Result<(), AppError> {
    info!("Initializing directory structure in {}...", path);
    let path = Path::new(&path);
    let base_path = Path::new(path);
    if base_path.exists() && fs::read_dir(base_path)?.next().is_some() {
        warn!("Directory already exists and is not empty. Skipping initialization...");
        return Ok(());
    }

    create_directory_if_not_exists(base_path);

    generate_template_file(
        OpenAPIGeneratorIgnore {},
        base_path,
        ".openapi-generator-ignore",
    )?;

    // Create directories
    create_directory_if_not_exists(base_path.join(CARGO_DIR).as_path());
    create_directory_if_not_exists(base_path.join(DEVCONTAINER_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_OPERATOR_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_CRDGEN_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_CRDGEN_DIR).join("src").as_path());
    create_directory_if_not_exists(base_path.join(K8S_TESTS_DIR).as_path());
    create_directory_if_not_exists(&base_path.join(K8S_OPERATOR_CONTROLLERS_DIR));
    create_directory_if_not_exists(&base_path.join(K8S_OPERATOR_TYPES_DIR));
    create_directory_if_not_exists(&base_path.join(K8S_TESTS_UTILS_DIR));
    create_directory_if_not_exists(base_path.join(K8S_MANIFESTS_CRDS_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_MANIFESTS_RBAC_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_MANIFESTS_OPERATOR_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_MANIFESTS_EXAMPLES_DIR).as_path());

    generate_template_file(CargoToml {}, base_path, "Cargo.toml")?;
    generate_template_file(
        OperatorCargoToml {},
        base_path.join(K8S_OPERATOR_DIR).as_path(),
        "Cargo.toml",
    )?;
    generate_template_file(
        CrdgenCargoToml {},
        base_path.join(K8S_CRDGEN_DIR).as_path(),
        "Cargo.toml",
    )?;
    generate_template_file(
        TestsCargoToml {},
        base_path.join(K8S_TESTS_DIR).as_path(),
        "Cargo.toml",
    )?;

    // Generate main files
    generate_template_file(
        OperatorMain {
            api_group: "example.com".to_string(),
            api_version: "v1".to_string(),
            controllers: vec![],
            types: vec![],
        },
        base_path.join(K8S_OPERATOR_DIR).join("src").as_path(),
        "main.rs",
    )?;
    generate_template_file(
        CrdgenMain {
            resources: BTreeMap::new(),
        },
        base_path.join(K8S_CRDGEN_DIR).join("src").as_path(),
        "main.rs",
    )?;
    generate_template_file(
        TestsMain {},
        base_path.join(K8S_TESTS_DIR).as_path(),
        "main.rs",
    )?;

    let project_name: String = "Example Operator Project".to_string();

    // Generate operator files
    generate_template_file(
        Cli {
            project_name: project_name.clone(),
            version: "0.1.0".to_string(),
            author: "example".to_string(),
        },
        base_path.join(K8S_OPERATOR_DIR).join("src").as_path(),
        "cli.rs",
    )?;

    // Generate root files
    generate_template_file(Dockerignore {}, base_path, ".dockerignore")?;
    generate_template_file(Editorconfig {}, base_path, ".editorconfig")?;
    generate_template_file(EnvExample {}, base_path, ".env.example")?;
    generate_template_file(GitAttributes {}, base_path, ".gitattributes")?;
    generate_template_file(GitIgnore {}, base_path, ".gitignore")?;
    generate_template_file(Taskfile {}, base_path, "Taskfile.yml")?;
    generate_template_file(Prettierrc {}, base_path, ".prettierrc.yaml")?;
    generate_template_file(RustfmtToml {}, base_path, ".rustfmt.toml")?;
    generate_template_file(ClusterYaml {}, base_path, "Cluster.yaml")?;
    generate_template_file(Dockerfile {}, base_path, "Dockerfile")?;
    generate_template_file(ReadmeMd { project_name }, base_path, "README.md")?;

    // Generate devcontainer files
    generate_template_file(
        Json {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        "devcontainer.json",
    )?;
    generate_template_file(
        Deps {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        "deps.sh",
    )?;
    generate_template_file(
        SetupGit {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        "setup-git.sh",
    )?;
    generate_template_file(
        LaunchJsonExample {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        "launch.json",
    )?;
    generate_template_file(
        Zshrc {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        ".zshrc",
    )?;

    generate_template_file(
        CargoConfig {},
        base_path.join(CARGO_DIR).as_path(),
        "config.toml",
    )?;

    create_file_if_not_exists(
        base_path.join(K8S_OPERATOR_CONTROLLERS_DIR).as_path(),
        "mod.rs",
    );
    create_file_if_not_exists(base_path.join(K8S_OPERATOR_TYPES_DIR).as_path(), "mod.rs");
    create_file_if_not_exists(base_path.join(K8S_TESTS_UTILS_DIR).as_path(), "mod.rs");

    let tests_utils_path_buf = base_path.join(K8S_TESTS_UTILS_DIR);
    let tests_utils_path: &Path = tests_utils_path_buf.as_path();
    generate_template_file(UtilsClient {}, tests_utils_path, "client.rs")?;
    generate_template_file(UtilsOperator {}, tests_utils_path, "operator.rs")?;
    generate_template_file(UtilsCluster {}, tests_utils_path, "cluster.rs")?;

    add_tests_util_to_modfile(base_path, "client");
    add_tests_util_to_modfile(base_path, "operator");
    add_tests_util_to_modfile(base_path, "cluster");

    set_executable_permission(base_path.join(DEVCONTAINER_DIR).join("deps.sh").as_path());
    set_executable_permission(
        base_path
            .join(DEVCONTAINER_DIR)
            .join("setup-git.sh")
            .as_path(),
    );
    info!("Initialized project at {}", path.display());
    Ok(())
}