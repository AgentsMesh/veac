use super::*;
use crate::program::expression::ast::ExpressionKind;

fn parsed(source: &str) -> Result<Expression, ExpressionError> {
    crate::program::expression::parser::parse(crate::program::expression::lexer::lex(source)?)
}

#[test]
fn for_retains_binding_iterable_body_and_complete_span() {
    let source = "for value in 0 .. 3 { value + 1 }";
    let expression = parsed(source).unwrap();
    assert_eq!(expression.span, 0..source.len());
    let ExpressionKind::For {
        binding,
        binding_span,
        iterable,
        body,
    } = expression.kind
    else {
        panic!("expected for expression");
    };
    assert_eq!(binding, "value");
    assert_eq!(&source[binding_span], "value");
    assert!(matches!(iterable.kind, ExpressionKind::Range { .. }));
    assert!(matches!(body.result.kind, ExpressionKind::Binary { .. }));
}

#[test]
fn for_requires_a_valid_binding_source_clause_and_body() {
    for (source, code) in [
        ("for in values { 1 }", "EXPRESSION_FOR_BINDING"),
        ("for value values { value }", "EXPRESSION_EXPECTED_TOKEN"),
        ("for value in values value", "EXPRESSION_EXPECTED_TOKEN"),
    ] {
        assert_eq!(parsed(source).unwrap_err().code(), code, "{source}");
    }
}
