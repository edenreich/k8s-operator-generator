use crate::errors::AppError;
use crate::templates::general::{
    CargoConfig, ClusterYaml, Dockerfile, Dockerignore, Editorconfig, EnvExample, GitAttributes,
    GitIgnore, Prettierrc, ReadmeMd, RustfmtToml, Taskfile,
};
use crate::templates::tests::{TestsMain, TestsUtilsClient, TestsUtilsCluster, TestsUtilsOperator};
use crate::templates::{
    cargo::{CargoToml, K8sCrdgenCargoToml, K8sOperatorCargoToml},
    devcontainer::{
        DevcontainerDeps, DevcontainerJson, DevcontainerLaunchJsonExample, DevcontainerSetupGit,
        DevcontainerZshrc,
    },
    general::OpenAPIGeneratorIgnore,
    tests::K8sTestsCargoToml,
};
use crate::utils::{
    add_tests_util_to_modfile, create_directory_if_not_exists, create_file_if_not_exists,
    generate_template_file, set_executable_permission,
};
use log::{info, warn};
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

    create_directory_if_not_exists(base_path.join(CARGO_DIR).as_path());
    create_directory_if_not_exists(base_path.join(DEVCONTAINER_DIR).as_path());

    create_directory_if_not_exists(base_path.join(K8S_OPERATOR_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_CRDGEN_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_CRDGEN_DIR).join("src").as_path());
    create_directory_if_not_exists(base_path.join(K8S_TESTS_DIR).as_path());

    generate_template_file(CargoToml {}, base_path, "Cargo.toml")?;
    generate_template_file(
        K8sOperatorCargoToml {},
        base_path.join(K8S_OPERATOR_DIR).as_path(),
        "Cargo.toml",
    )?;
    generate_template_file(
        K8sCrdgenCargoToml {},
        base_path.join(K8S_CRDGEN_DIR).as_path(),
        "Cargo.toml",
    )?;
    generate_template_file(
        K8sTestsCargoToml {},
        base_path.join(K8S_TESTS_DIR).as_path(),
        "Cargo.toml",
    )?;

    generate_template_file(Dockerignore {}, base_path, ".dockerignore")?;
    generate_template_file(Editorconfig {}, base_path, ".editorconfig")?;
    generate_template_file(EnvExample {}, base_path, ".env.example")?;
    generate_template_file(GitAttributes {}, base_path, ".gitattributes")?;
    generate_template_file(GitIgnore {}, base_path, ".gitignore")?;
    generate_template_file(Taskfile {}, base_path, "Taskfile.yml")?;
    generate_template_file(ReadmeMd {}, base_path, "README.md")?;
    generate_template_file(Prettierrc {}, base_path, ".prettierrc")?;
    generate_template_file(RustfmtToml {}, base_path, ".rustfmt.toml")?;
    generate_template_file(ClusterYaml {}, base_path, "Cluster.yaml")?;
    generate_template_file(Dockerfile {}, base_path, "Dockerfile")?;

    generate_template_file(
        DevcontainerJson {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        "devcontainer.json",
    )?;
    generate_template_file(
        DevcontainerDeps {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        "deps.sh",
    )?;
    generate_template_file(
        DevcontainerSetupGit {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        "setup-git.sh",
    )?;
    generate_template_file(
        DevcontainerLaunchJsonExample {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        "launch.json",
    )?;
    generate_template_file(
        DevcontainerZshrc {},
        base_path.join(DEVCONTAINER_DIR).as_path(),
        ".zshrc",
    )?;

    generate_template_file(
        CargoConfig {},
        base_path.join(CARGO_DIR).as_path(),
        "config.toml",
    )?;

    create_directory_if_not_exists(&base_path.join(K8S_OPERATOR_CONTROLLERS_DIR));
    create_directory_if_not_exists(&base_path.join(K8S_OPERATOR_TYPES_DIR));
    create_directory_if_not_exists(&base_path.join(K8S_TESTS_UTILS_DIR));

    create_file_if_not_exists(
        base_path.join(K8S_OPERATOR_CONTROLLERS_DIR).as_path(),
        "mod.rs",
    );
    create_file_if_not_exists(base_path.join(K8S_OPERATOR_TYPES_DIR).as_path(), "mod.rs");
    create_file_if_not_exists(base_path.join(K8S_TESTS_UTILS_DIR).as_path(), "mod.rs");

    let tests_utils_path_buf = base_path.join(K8S_TESTS_UTILS_DIR);
    let tests_utils_path: &Path = tests_utils_path_buf.as_path();
    generate_template_file(TestsUtilsClient {}, tests_utils_path, "client.rs")?;
    generate_template_file(TestsUtilsOperator {}, tests_utils_path, "operator.rs")?;
    generate_template_file(TestsUtilsCluster {}, tests_utils_path, "cluster.rs")?;

    add_tests_util_to_modfile(base_path, "client");
    add_tests_util_to_modfile(base_path, "operator");
    add_tests_util_to_modfile(base_path, "cluster");

    // TODO - make the tests generated dynamically based on the defined API controllers and types.
    generate_template_file(
        TestsMain {},
        base_path.join(K8S_TESTS_DIR).as_path(),
        "main.rs",
    )?;

    create_directory_if_not_exists(base_path.join(K8S_MANIFESTS_CRDS_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_MANIFESTS_RBAC_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_MANIFESTS_OPERATOR_DIR).as_path());
    create_directory_if_not_exists(base_path.join(K8S_MANIFESTS_EXAMPLES_DIR).as_path());

    set_executable_permission(base_path.join(DEVCONTAINER_DIR).join("deps.sh").as_path());
    set_executable_permission(
        base_path
            .join(DEVCONTAINER_DIR)
            .join("setup-git.sh")
            .as_path(),
    );
    println!("Initialized project at {}", path.display());
    Ok(())
}
