use askama::Template;
use insta::assert_snapshot;
use kopgen::{errors::AppError, templates::general::EnvExample};

#[test]
fn render_env_example() -> Result<(), AppError> {
    let template = EnvExample {
        operator_name: "example_operator".to_string(),
        operator_author: "Author Name".to_string(),
        operator_api_group: "example.com".to_string(),
        operator_api_version: "v1".to_string(),
        operator_resource_ref: "resourceRef".to_string(),
        operator_example_metadata_spec_field_ref: "fieldRef".to_string(),
        operator_include_tags: "tag1".to_string(),
        operator_secret_name: "secret".to_string(),
    };
    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}

#[test]
fn render_env_example_with_empty_string() -> Result<(), AppError> {
    let template = EnvExample {
        operator_name: "".to_string(),
        operator_author: "".to_string(),
        operator_api_group: "".to_string(),
        operator_api_version: "".to_string(),
        operator_resource_ref: "".to_string(),
        operator_example_metadata_spec_field_ref: "".to_string(),
        operator_include_tags: "".to_string(),
        operator_secret_name: "".to_string(),
    };
    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}
