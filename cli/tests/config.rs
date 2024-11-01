use kopgen::config::{ConfigProvider, EnvConfigProvider};
use serial_test::serial;
use std::env;

fn setup_env() {
    env::set_var(
        "OPENAPI_DOWNLOAD_URL",
        "https://github.com/edenreich/kopgen/blob/main/openapi.yaml",
    );
    env::set_var("INSTALL_CRDS", "true");
    env::set_var("RUST_LOG", "info");
    env::set_var("RUST_BACKTRACE", "1");
    env::set_var("CONTAINER_REGISTRY", "localhost:5005");
    env::set_var("CLUSTER_NAME", "k3d-k3s-default");
    env::set_var("TARGET_ARCH", "aarch64-unknown-linux-musl");
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
fn test_config_from_env() {
    setup_env();

    let config = EnvConfigProvider::load_config().expect("Failed to load configuration");

    assert_eq!(config.kubernetes_operator_group, "test-group");
    assert_eq!(config.kubernetes_operator_version, "v1");
    assert_eq!(config.kubernetes_operator_resource_ref, "test-resource-ref");
    assert_eq!(
        config.kubernetes_operator_include_tags,
        vec!["tag1", "tag2"]
    );
    assert_eq!(
        config.kubernetes_operator_example_metadata_spec_field_ref,
        "test-field-ref"
    );

    clear_env();
}

#[test]
#[serial]
fn test_config_missing_required_env_vars() {
    clear_env();

    assert!(env::var("KUBERNETES_OPERATOR_GROUP").is_err());

    let _: Option<()> = match EnvConfigProvider::load_config() {
        Ok(_) => None,
        Err(e) => assert_eq!(e, "KUBERNETES_OPERATOR_GROUP environment variable not set").into(),
    };
}

#[test]
#[serial]
fn test_config_parses_comma_separated_tags_as_vector() {
    setup_env();

    let config = EnvConfigProvider::load_config().expect("Failed to load configuration");

    assert_eq!(
        config.kubernetes_operator_include_tags,
        vec!["tag1", "tag2"]
    );

    clear_env();
}
