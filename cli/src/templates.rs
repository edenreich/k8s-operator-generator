use askama::Template;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod filters {
    pub fn dashcase<T: std::fmt::Display>(s: T) -> ::askama::Result<String> {
        Ok(s.to_string()
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join("-")
            .to_lowercase())
    }
}

// Common Identifiers and Structs
pub struct RoleTemplateIdentifiers {
    pub api_group: String,
    pub resources: Vec<String>,
}

pub struct ClusterRoleTemplateIdentifiers {
    pub api_group: String,
    pub resources: Vec<String>,
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

pub struct Field {
    pub pub_name: String,
    pub field_type: String,
}

pub struct ControllerAttributes {
    pub operation_id: String,
    pub http_method: String,
    pub action_summary: String,
}

// Tests Templates
pub mod tests {
    use super::*;

    #[derive(Template)]
    #[template(path = "tests/main.rs.jinja")]
    pub struct Main {}

    #[derive(Template)]
    #[template(path = "tests/utils_client.rs.jinja")]
    pub struct UtilsClient {}

    #[derive(Template)]
    #[template(path = "tests/utils_operator.rs.jinja")]
    pub struct UtilsOperator {}

    #[derive(Template)]
    #[template(path = "tests/utils_cluster.rs.jinja")]
    pub struct UtilsCluster {}
}

// Devcontainer Templates
pub mod devcontainer {
    use super::*;

    #[derive(Template)]
    #[template(path = ".devcontainer/devcontainer.json.jinja")]
    pub struct Json {}

    #[derive(Template)]
    #[template(path = ".devcontainer/dockerfile.jinja")]
    pub struct Dockerfile {}

    #[derive(Template)]
    #[template(path = ".devcontainer/setup-git.sh.jinja")]
    pub struct SetupGit {}

    #[derive(Template)]
    #[template(path = ".devcontainer/launch.json.jinja")]
    pub struct LaunchJsonExample {}

    #[derive(Template)]
    #[template(path = ".devcontainer/zshrc.jinja")]
    pub struct Zshrc {}
}

// Manifests Templates
pub mod manifests {
    use super::*;

    pub mod rbac {
        use super::*;

        #[derive(Template)]
        #[template(path = "manifests/rbac_role.yaml.jinja")]
        pub struct Role {
            pub identifiers: RoleTemplateIdentifiers,
        }

        #[derive(Template)]
        #[template(path = "manifests/rbac_cluster_role.yaml.jinja")]
        pub struct ClusterRole {
            pub identifiers: ClusterRoleTemplateIdentifiers,
        }

        #[derive(Template)]
        #[template(path = "manifests/rbac_service_account.yaml.jinja")]
        pub struct ServiceAccount {}

        #[derive(Template)]
        #[template(path = "manifests/rbac_role_binding.yaml.jinja")]
        pub struct RoleBinding {}

        #[derive(Template)]
        #[template(path = "manifests/rbac_cluster_role_binding.yaml.jinja")]
        pub struct ClusterRoleBinding {}
    }

    pub mod operator {
        use super::*;
        #[derive(Template)]
        #[template(path = "manifests/operator_deployment.yaml.jinja")]
        pub struct Deployment {
            pub secret_name: String,
        }

        #[derive(Template)]
        #[template(path = "manifests/operator_secret.yaml.jinja")]
        pub struct Secret {
            pub secret_name: String,
        }
    }

    pub mod examples {
        use super::*;
        #[derive(Template, Deserialize, Serialize)]
        #[template(path = "manifests/example.yaml.jinja")]
        pub struct Example {
            pub resources: Vec<Resource>,
        }
    }
}

// Operator Templates
pub mod operator {
    use super::*;

    #[derive(Template)]
    #[template(path = "operator/main.rs.jinja")]
    pub struct Main {
        pub api_group: String,
        pub api_version: String,
        pub controllers: Vec<String>,
        pub types: Vec<String>,
    }

    #[derive(Template)]
    #[template(path = "operator/cli.rs.jinja")]
    pub struct Cli {
        pub project_name: String,
        pub version: String,
        pub author: String,
    }

    #[derive(Template)]
    #[template(path = "operator/controller.rs.jinja")]
    pub struct Controller {
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
    pub struct ControllerActionDelete<'a> {
        pub arg_name: String,
        pub kind_struct: String,
        pub controllers: Vec<&'a ControllerAttributes>,
        pub resource_remote_ref: String,
    }

    #[derive(Template)]
    #[template(path = "operator/controller_action_update.jinja")]
    pub struct ControllerActionPut<'a> {
        pub arg_name: String,
        pub kind_struct: String,
        pub controllers: Vec<&'a ControllerAttributes>,
        pub resource_remote_ref: String,
    }

    #[derive(Template)]
    #[template(path = "operator/controller_action_create.jinja")]
    pub struct ControllerActionPost<'a> {
        pub arg_name: String,
        pub kind_struct: String,
        pub controllers: Vec<&'a ControllerAttributes>,
        pub resource_remote_ref: String,
    }

    #[derive(Template)]
    #[template(path = "operator/type.rs.jinja")]
    pub struct Type {
        pub tag_name: String,
        pub type_name: String,
        pub api_version: String,
        pub group_name: String,
        pub fields: Vec<Field>,
        pub reference_id: String,
    }

    #[derive(Template)]
    #[template(path = "operator/lib.rs.jinja")]
    pub struct Lib {}
}

// Cargo Templates
pub mod cargo {
    use super::*;

    #[derive(Template)]
    #[template(path = "cargo.toml.jinja")]
    pub struct CargoToml {}

    #[derive(Template)]
    #[template(path = "operator/cargo.toml.jinja")]
    pub struct OperatorCargoToml {}

    #[derive(Template)]
    #[template(path = "crdgen/cargo.toml.jinja")]
    pub struct CrdgenCargoToml {}

    #[derive(Template)]
    #[template(path = "tests/cargo.toml.jinja")]
    pub struct TestsCargoToml {}
}

// Crdgen Templates
pub mod crdgen {
    use super::*;

    #[derive(Template)]
    #[template(path = "crdgen/main.rs.jinja")]
    pub struct Main {
        pub resources: BTreeMap<String, String>,
    }
}

// General Templates
pub mod general {
    use super::*;

    #[derive(Template)]
    #[template(path = ".dockerignore.jinja")]
    pub struct Dockerignore {}

    #[derive(Template)]
    #[template(path = ".editorconfig.jinja")]
    pub struct Editorconfig {}

    #[derive(Template)]
    #[template(path = ".env.example.jinja")]
    pub struct EnvExample {
        pub operator_name: String,
        pub operator_author: String,
        pub operator_api_group: String,
        pub operator_api_version: String,
        pub operator_resource_ref: String,
        pub operator_example_metadata_spec_field_ref: String,
        pub operator_include_tags: String,
        pub operator_secret_name: String,
    }

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
    #[template(path = "cluster.yaml.jinja")]
    pub struct ClusterYaml {}

    #[derive(Template)]
    #[template(path = "dockerfile.jinja")]
    pub struct Dockerfile {}

    #[derive(Template)]
    #[template(path = "readme.md.jinja")]
    pub struct ReadmeMd {
        pub project_name: String,
    }

    #[derive(Template)]
    #[template(path = "taskfile.yml.jinja")]
    pub struct Taskfile {}

    #[derive(Template)]
    #[template(path = ".cargo/config.toml.jinja")]
    pub struct CargoConfig {}
}
