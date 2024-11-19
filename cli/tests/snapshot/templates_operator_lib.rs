use askama::Template;
use insta::assert_snapshot;
use kopgen::{errors::AppError, templates::operator::Lib};

#[test]
fn render() -> Result<(), AppError> {
    let template = Lib {};

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}
