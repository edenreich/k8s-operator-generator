use askama::Template;
use insta::assert_snapshot;
use kopgen::{
    errors::AppError,
    templates::{operator::Type, Field},
};

#[test]
fn render_basic_type() -> Result<(), AppError> {
    let template = Type {
        tag_name: "basic".to_string(),
        api_version: "v1".to_string(),
        group_name: "example.com".to_string(),
        reference_id: "basic-type-001".to_string(),
        type_name: "BasicType".to_string(),
        fields: vec![
            Field {
                pub_name: "id".to_string(),
                field_type: "i32".to_string(),
            },
            Field {
                pub_name: "name".to_string(),
                field_type: "String".to_string(),
            },
        ],
    };

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}

#[test]
fn render_complex_type() -> Result<(), AppError> {
    let template = Type {
        tag_name: "complex".to_string(),
        api_version: "v1".to_string(),
        group_name: "example.com".to_string(),
        reference_id: "complex-type-001".to_string(),
        type_name: "ComplexType".to_string(),
        fields: vec![
            Field {
                pub_name: "id".to_string(),
                field_type: "i32".to_string(),
            },
            Field {
                pub_name: "details".to_string(),
                field_type: "Details".to_string(),
            },
            Field {
                pub_name: "tags".to_string(),
                field_type: "Vec<String>".to_string(),
            },
        ],
    };

    let rendered = template.render()?;
    assert_snapshot!(rendered);
    Ok(())
}
