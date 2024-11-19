use askama::Template;
use insta::assert_snapshot;
use kopgen::templates::cargo::OperatorCargoToml;

#[test]
fn render() {
    let template = OperatorCargoToml {};

    let rendered = template.render().expect("Failed to render template");
    assert_snapshot!(rendered);
}
