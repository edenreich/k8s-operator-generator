use askama::Template;
use insta::assert_snapshot;
use kopgen::{errors::AppError, templates::operator::Cli};

#[test]
fn render() -> Result<(), AppError> {
    let template = Cli {
        project_name: "Operator Example".to_string(),
        author: "Jane Doe".to_string(),
        version: "0.1.0".to_string(),
    };

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}
