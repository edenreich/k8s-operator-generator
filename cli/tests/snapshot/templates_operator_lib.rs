use askama::Template;
use insta::assert_snapshot;
use kopgen::templates::operator::Lib;

#[test]
fn render() {
    let template = Lib {};

    let rendered = template.render().expect("Failed to render template");
    assert_snapshot!(rendered);
}
