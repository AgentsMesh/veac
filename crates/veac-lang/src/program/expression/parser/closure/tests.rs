use super::*;
use crate::program::expression::ast::ExpressionKind;

fn parsed(source: &str) -> Result<Expression, ExpressionError> {
    super::super::parse(crate::program::expression::lexer::lex(source)?)
}

#[test]
fn closure_retains_explicit_signature_body_and_span() {
    let source = "fn(value: int) -> int effect pure { value }";
    let expression = parsed(source).unwrap();
    assert_eq!(expression.span, 0..source.len());
    let ExpressionKind::Closure {
        parameters,
        return_type,
        body,
        ..
    } = expression.kind
    else {
        panic!("expected closure");
    };
    assert_eq!(parameters.len(), 1);
    assert_eq!(parameters[0].name, "value");
    assert_eq!(parameters[0].annotation.syntax.to_string(), "int");
    assert_eq!(return_type.syntax.to_string(), "int");
    assert!(matches!(body.result.kind, ExpressionKind::Symbol(_)));
}

#[test]
fn closure_retains_structured_qualified_nominal_annotations() {
    let source = "fn(card: brand.Card, mood: Mood) -> brand.Card effect pure { card }";
    let expression = parsed(source).unwrap();
    let ExpressionKind::Closure {
        parameters,
        return_type,
        ..
    } = expression.kind
    else {
        panic!("expected closure");
    };
    assert_eq!(parameters[0].annotation.syntax.to_string(), "brand.Card");
    assert_eq!(parameters[1].annotation.syntax.to_string(), "Mood");
    assert_eq!(return_type.syntax.to_string(), "brand.Card");
    assert_eq!(&source[parameters[0].annotation.span.clone()], "brand.Card");
}

#[test]
fn every_primary_supports_repeated_postfix_calls() {
    let expression = parsed("(fn(x: int) -> int effect pure { x })(1)(2)").unwrap();
    let ExpressionKind::Call { callee, arguments } = expression.kind else {
        panic!("expected outer call");
    };
    assert_eq!(arguments.len(), 1);
    assert!(matches!(callee.kind, ExpressionKind::Call { .. }));

    let returned = parsed("factory()(1)").unwrap();
    let ExpressionKind::Call { callee, .. } = returned.kind else {
        panic!("expected returned closure call");
    };
    assert!(matches!(callee.kind, ExpressionKind::Call { .. }));
}

#[test]
fn closure_parameters_are_closed_and_explicit() {
    for (source, code) in [
        (
            "fn(value) -> int effect pure { 1 }",
            "EXPRESSION_EXPECTED_TOKEN",
        ),
        (
            "fn(value: int, value: int) -> int effect pure { value }",
            "EXPRESSION_DUPLICATE_PARAMETER",
        ),
        (
            "fn(value: int,) -> int effect pure { value }",
            "EXPRESSION_EXPECTED_VALUE",
        ),
        (
            "fn(value: int) -> int effect unknown { value }",
            "EXPRESSION_TYPE_SYNTAX",
        ),
        ("fn(value: int) int { value }", "EXPRESSION_EXPECTED_TOKEN"),
        (
            "fn(value: brand.) -> brand.Card effect pure { value }",
            "EXPRESSION_TYPE_SYNTAX",
        ),
    ] {
        assert_eq!(parsed(source).unwrap_err().code(), code, "{source}");
    }
}

#[test]
fn closure_annotation_parser_enforces_type_depth_and_arity_limits() {
    let accepted_type = format!(
        "{}int{}",
        "list<".repeat(crate::program::expression::MAX_VALUE_TYPE_DEPTH - 1),
        ">".repeat(crate::program::expression::MAX_VALUE_TYPE_DEPTH - 1)
    );
    parsed(&format!(
        "fn(value: {accepted_type}) -> int effect pure {{ 1 }}"
    ))
    .unwrap();

    let rejected_type = format!(
        "{}int{}",
        "list<".repeat(crate::program::expression::MAX_VALUE_TYPE_DEPTH),
        ">".repeat(crate::program::expression::MAX_VALUE_TYPE_DEPTH)
    );
    let error = parsed(&format!(
        "fn(value: {rejected_type}) -> int effect pure {{ 1 }}"
    ))
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TYPE_SYNTAX");
    assert!(error.message().contains("depth limit"));

    let tuple = std::iter::repeat_n("int", crate::program::expression::MAX_VALUE_TYPE_ARITY + 1)
        .collect::<Vec<_>>()
        .join(", ");
    let error = parsed(&format!("fn(value: ({tuple})) -> int effect pure {{ 1 }}")).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TYPE_SYNTAX");
    assert!(error.message().contains("arity limit"));
}
