use askama::Template;
use insta::assert_snapshot;
use kopgen::{errors::AppError, templates::cargo::OperatorCargoToml};

#[test]
fn render() -> Result<(), AppError> {
    let template = OperatorCargoToml {};

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}
