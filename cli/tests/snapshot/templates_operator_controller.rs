use askama::Template;
use insta::assert_snapshot;
use kopgen::templates::{operator::Controller, Field};

#[test]
fn render() {
    let template = Controller {
        tag: "example_tag".to_string(),
        arg_name: "argName".to_string(),
        kind_struct: "ExampleKind".to_string(),
        dto_fields: vec![
            Field {
                pub_name: "field1".to_string(),
                field_type: "string".to_string(),
            },
            Field {
                pub_name: "field2".to_string(),
                field_type: "string".to_string(),
            },
        ],
        resource_remote_ref: "resourceRef".to_string(),
        has_create_action: true,
        has_update_action: true,
        has_delete_action: false,
        api_url: "https://api.example.com".to_string(),
    };
    let rendered = template.render().expect("Failed to render template");
    assert_snapshot!(rendered);
}
