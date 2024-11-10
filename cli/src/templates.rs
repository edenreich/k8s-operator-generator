use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "k8s_tests_main.jinja")]
pub struct TestsMain {}

#[derive(Template)]
#[template(path = "k8s_tests_utils_client.jinja")]
pub struct TestsUtilsClient {}

#[derive(Template)]
#[template(path = "k8s_tests_utils_operator.jinja")]
pub struct TestsUtilsOperator {}
#[derive(Template)]
#[template(path = "k8s_tests_utils_cluster.jinja")]

pub struct TestsUtilsCluster {}
#[derive(Template)]
#[template(path = ".dockerignore.jinja")]
pub struct Dockerignore {}

#[derive(Template)]
#[template(path = ".editorconfig.jinja")]
pub struct Editorconfig {}

#[derive(Template)]
#[template(path = ".env.example.jinja")]
pub struct EnvExample {}

#[derive(Template)]
#[template(path = ".gitattributes.jinja")]
pub struct GitAttributes {}

#[derive(Template)]
#[template(path = ".gitignore.jinja")]
pub struct GitIgnore {}

#[derive(Template)]
#[template(path = ".openapi-generator-ignore.jinja")]
pub struct OpenAPIGeneratorIgnore {}

#[derive(Template)]
#[template(path = ".prettierrc.jinja")]
pub struct Prettierrc {}

#[derive(Template)]
#[template(path = ".rustfmt.toml.jinja")]
pub struct RustfmtToml {}

#[derive(Template)]
#[template(path = "cargo.toml.jinja")]
pub struct CargoToml {}

#[derive(Template)]
#[template(path = "cluster.yaml.jinja")]
pub struct ClusterYaml {}

#[derive(Template)]
#[template(path = "dockerfile.jinja")]
pub struct Dockerfile {}

#[derive(Template)]
#[template(path = "readme.md.jinja")]
pub struct ReadmeMd {}

#[derive(Template)]
#[template(path = "taskfile.jinja")]
pub struct Taskfile {}

#[derive(Template)]
#[template(path = ".devcontainer_devcontainer.json.jinja")]
pub struct DevcontainerJson {}

#[derive(Template)]
#[template(path = ".devcontainer_deps.sh.jinja")]
pub struct DevcontainerDeps {}

#[derive(Template)]
#[template(path = ".devcontainer_setup-git.sh.jinja")]
pub struct DevcontainerSetupGit {}

#[derive(Template)]
#[template(path = ".devcontainer_launch.json.jinja")]
pub struct DevcontainerLaunchJsonExample {}

#[derive(Template)]
#[template(path = ".devcontainer_zshrc.jinja")]
pub struct DevcontainerZshrc {}

#[derive(Template)]
#[template(path = ".cargo_config.toml.jinja")]
pub struct CargoConfig {}

#[derive(Template)]
#[template(path = "k8s_operator_cargo.toml.jinja")]
pub struct K8sOperatorCargoToml {}

#[derive(Template)]
#[template(path = "k8s_crdgen_cargo.toml.jinja")]
pub struct K8sCrdgenCargoToml {}

#[derive(Template)]
#[template(path = "k8s_tests_cargo.toml.jinja")]
pub struct K8sTestsCargoToml {}

pub struct RoleTemplateIdentifiers {
    pub api_group: String,
    pub resources: Vec<String>,
}

#[derive(Template)]
#[template(path = "manifest_rbac_role.jinja")]
pub struct RoleTemplate {
    pub identifiers: RoleTemplateIdentifiers,
}

pub struct ClusterRoleTemplateIdentifiers {
    pub api_group: String,
    pub resources: Vec<String>,
}

#[derive(Template)]
#[template(path = "manifest_rbac_cluster_role.jinja")]
pub struct ClusterRoleTemplate {
    pub identifiers: ClusterRoleTemplateIdentifiers,
}

#[derive(Template)]
#[template(path = "manifest_rbac_service_account.jinja")]
pub struct ServiceAccountTemplate {}

#[derive(Template)]
#[template(path = "manifest_rbac_role_binding.jinja")]
pub struct RoleBindingTemplate {}

#[derive(Template)]
#[template(path = "manifest_rbac_cluster_role_binding.jinja")]
pub struct ClusterRoleBindingTemplate {}

#[derive(Template)]
#[template(path = "manifest_operator_deployment.jinja")]
pub struct OperatorDeploymentTemplate {}

#[derive(Template)]
#[template(path = "manifest_operator_secret.jinja")]
pub struct OperatorSecretTemplate {}

#[derive(Template, Deserialize, Serialize)]
#[template(path = "manifest_example.jinja")]
pub struct ExampleTemplate {
    pub resources: Vec<Resource>,
}

#[derive(Serialize, Deserialize)]
pub struct Resource {
    pub api_group: String,
    pub api_version: String,
    pub kind: String,
    pub metadata: Metadata,
    pub spec: String,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
}
