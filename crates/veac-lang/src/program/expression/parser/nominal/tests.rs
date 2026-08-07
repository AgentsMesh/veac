use super::*;
use crate::program::expression::ast::ExpressionKind;

fn parsed(source: &str) -> Result<Expression, ExpressionError> {
    super::super::parse(crate::program::expression::lexer::lex(source)?)
}

#[test]
fn construction_retains_path_authored_field_order_and_spans() {
    let source = "module.Card { second: effect(2), first: 1 }";
    let expression = parsed(source).unwrap();
    assert_eq!(expression.span, 0..source.len());
    let ExpressionKind::NominalConstruct { path, fields } = expression.kind else {
        panic!("expected nominal construction");
    };
    assert_eq!(
        path.iter()
            .map(|segment| segment.name.as_str())
            .collect::<Vec<_>>(),
        ["module", "Card"]
    );
    assert_eq!(
        fields
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        ["second", "first"]
    );
    assert_eq!(&source[fields[0].name_span.clone()], "second");
    assert!(matches!(fields[0].value.kind, ExpressionKind::Call { .. }));
}

#[test]
fn construction_accepts_empty_and_trailing_comma_forms() {
    for source in [
        "Empty {}",
        "Card { title: \"x\", }",
        "Card { title: \"x\", count: 2, }",
    ] {
        assert!(
            matches!(
                parsed(source).unwrap().kind,
                ExpressionKind::NominalConstruct { .. }
            ),
            "{source}"
        );
    }
}

#[test]
fn constructed_values_support_postfix_projection() {
    let expression = parsed("Card {}.title").unwrap();
    let ExpressionKind::FieldProject {
        receiver, field, ..
    } = expression.kind
    else {
        panic!("expected field projection");
    };
    assert_eq!(field, "title");
    assert!(matches!(
        receiver.kind,
        ExpressionKind::NominalConstruct { .. }
    ));
}

#[test]
fn duplicate_authored_fields_are_rejected_before_resolution() {
    let error = parsed("Card { title: 1, title: 2 }").unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_DUPLICATE_NOMINAL_FIELD");
}
