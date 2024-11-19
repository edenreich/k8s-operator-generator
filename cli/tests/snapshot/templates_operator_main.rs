use askama::Template;
use insta::assert_snapshot;
use kopgen::{errors::AppError, templates::operator::Main};

#[test]
fn render_with_controllers_and_types() -> Result<(), AppError> {
    let template = Main {
        api_group: "example.com".to_string(),
        api_version: "v1".to_string(),
        controllers: vec!["controller1".to_string(), "controller2".to_string()],
        types: vec!["Type1".to_string(), "Type2".to_string()],
    };

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}

#[test]
fn render_with_controllers() -> Result<(), AppError> {
    let template = Main {
        api_group: "example.com".to_string(),
        api_version: "v1".to_string(),
        controllers: vec!["controller1".to_string(), "controller2".to_string()],
        types: vec![],
    };

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}

#[test]
fn render_with_types() -> Result<(), AppError> {
    let template = Main {
        api_group: "example.com".to_string(),
        api_version: "v1".to_string(),
        controllers: vec![],
        types: vec!["Type1".to_string(), "Type2".to_string()],
    };

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}

#[test]
fn render_without_controllers_or_types() -> Result<(), AppError> {
    let template = Main {
        api_group: "example.com".to_string(),
        api_version: "v1".to_string(),
        controllers: vec![],
        types: vec![],
    };

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}
