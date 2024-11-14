use askama::Template;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Template)]
#[template(path = "tests/main.rs.jinja")]
pub struct TestsMain {}

#[derive(Template)]
#[template(path = "tests/utils_client.rs.jinja")]
pub struct TestsUtilsClient {}

#[derive(Template)]
#[template(path = "tests/utils_operator.rs.jinja")]
pub struct TestsUtilsOperator {}

#[derive(Template)]
#[template(path = "tests/utils_cluster.rs.jinja")]
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
#[template(path = ".prettierrc.yaml.jinja")]
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
#[template(path = "taskfile.yml.jinja")]
pub struct Taskfile {}

#[derive(Template)]
#[template(path = ".devcontainer/devcontainer.json.jinja")]
pub struct DevcontainerJson {}

#[derive(Template)]
#[template(path = ".devcontainer/deps.sh.jinja")]
pub struct DevcontainerDeps {}

#[derive(Template)]
#[template(path = ".devcontainer/setup-git.sh.jinja")]
pub struct DevcontainerSetupGit {}

#[derive(Template)]
#[template(path = ".devcontainer/launch.json.jinja")]
pub struct DevcontainerLaunchJsonExample {}

#[derive(Template)]
#[template(path = ".devcontainer/zshrc.jinja")]
pub struct DevcontainerZshrc {}

#[derive(Template)]
#[template(path = ".cargo/config.toml.jinja")]
pub struct CargoConfig {}

#[derive(Template)]
#[template(path = "operator/cargo.toml.jinja")]
pub struct K8sOperatorCargoToml {}

#[derive(Template)]
#[template(path = "crdgen/cargo.toml.jinja")]
pub struct K8sCrdgenCargoToml {}

#[derive(Template)]
#[template(path = "tests/cargo.toml.jinja")]
pub struct K8sTestsCargoToml {}

pub struct RoleTemplateIdentifiers {
    pub api_group: String,
    pub resources: Vec<String>,
}

#[derive(Template)]
#[template(path = "manifests/rbac_role.yaml.jinja")]
pub struct RoleTemplate {
    pub identifiers: RoleTemplateIdentifiers,
}

pub struct ClusterRoleTemplateIdentifiers {
    pub api_group: String,
    pub resources: Vec<String>,
}

#[derive(Template)]
#[template(path = "manifests/rbac_cluster_role.yaml.jinja")]
pub struct ClusterRoleTemplate {
    pub identifiers: ClusterRoleTemplateIdentifiers,
}

#[derive(Template)]
#[template(path = "manifests/rbac_service_account.yaml.jinja")]
pub struct ServiceAccountTemplate {}

#[derive(Template)]
#[template(path = "manifests/rbac_role_binding.yaml.jinja")]
pub struct RoleBindingTemplate {}

#[derive(Template)]
#[template(path = "manifests/rbac_cluster_role_binding.yaml.jinja")]
pub struct ClusterRoleBindingTemplate {}

#[derive(Template)]
#[template(path = "manifests/operator_deployment.yaml.jinja")]
pub struct OperatorDeploymentTemplate {}

#[derive(Template)]
#[template(path = "manifests/operator_secret.yaml.jinja")]
pub struct OperatorSecretTemplate {}

#[derive(Template, Deserialize, Serialize)]
#[template(path = "manifests/example.yaml.jinja")]
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

#[derive(Template)]
#[template(path = "operator/main.rs.jinja")]
pub struct MainTemplate {
    pub api_group: String,
    pub api_version: String,
    pub controllers: Vec<String>,
    pub types: Vec<String>,
}

pub struct Field {
    pub pub_name: String,
    pub field_type: String,
}

#[derive(Template)]
#[template(path = "operator/controller.rs.jinja")]
pub struct ControllerTemplate {
    pub tag: String,
    pub arg_name: String,
    pub kind_struct: String,
    pub dto_fields: Vec<Field>,
    pub resource_remote_ref: String,
    pub has_create_action: bool,
    pub has_update_action: bool,
    pub has_delete_action: bool,
    pub api_url: String,
}

#[derive(Template)]
#[template(path = "operator/controller_action_delete.jinja")]
pub struct ControllerActionDeleteTemplate<'a> {
    pub arg_name: String,
    pub kind_struct: String,
    pub controllers: Vec<&'a ControllerAttributes>,
    pub resource_remote_ref: String,
}

#[derive(Template)]
#[template(path = "operator/controller_action_update.jinja")]
pub struct ControllerActionPutTemplate<'a> {
    pub arg_name: String,
    pub kind_struct: String,
    pub controllers: Vec<&'a ControllerAttributes>,
    pub resource_remote_ref: String,
}

#[derive(Template)]
#[template(path = "operator/controller_action_create.jinja")]
pub struct ControllerActionPostTemplate<'a> {
    pub arg_name: String,
    pub kind_struct: String,
    pub controllers: Vec<&'a ControllerAttributes>,
    pub resource_remote_ref: String,
}

pub struct ControllerAttributes {
    pub operation_id: String,
    pub http_method: String,
    pub action_summary: String,
}

#[derive(Template)]
#[template(path = "operator/type.rs.jinja")]
pub struct TypeTemplate {
    pub tag_name: String,
    pub type_name: String,
    pub api_version: String,
    pub group_name: String,
    pub fields: Vec<Field>,
    pub reference_id: String,
}

#[derive(Template)]
#[template(path = "operator/lib.rs.jinja")]
pub struct LibTemplate {}

#[derive(Template)]
#[template(path = "crdgen/main.rs.jinja")]
pub struct CrdGenTemplate {
    pub resources: BTreeMap<String, String>,
}
