use super::error;
use crate::program::expression::{evaluate, Environment};

#[test]
fn lexer_errors_are_stable_and_located() {
    for (source, code) in [
        ("$", "EXPRESSION_LEX_CHARACTER"),
        ("1..2", "EXPRESSION_NUMBER_LITERAL"),
        ("foo.", "EXPRESSION_SYMBOL"),
        ("1 + /* open", "EXPRESSION_BLOCK_COMMENT"),
        (r#""unterminated"#, "EXPRESSION_STRING_LITERAL"),
        (r#""bad\q""#, "EXPRESSION_STRING_ESCAPE"),
        (r#""bad\"#, "EXPRESSION_STRING_LITERAL"),
        (r#""\u1234""#, "EXPRESSION_STRING_ESCAPE"),
        (r#""\u{}""#, "EXPRESSION_STRING_ESCAPE"),
        (r#""\u{110000}""#, "EXPRESSION_STRING_ESCAPE"),
        ("#abcd", "EXPRESSION_COLOR_LITERAL"),
    ] {
        let error = error(source);
        assert_eq!(error.code(), code);
        assert!(error.span().end <= source.len());
        assert!(!error.message().is_empty());
        assert!(error.to_string().contains(code));
        let as_error: &dyn std::error::Error = &error;
        assert!(as_error.source().is_none());
    }

    let raw_control = "\"line\nfeed\"";
    assert_eq!(error(raw_control).code(), "EXPRESSION_STRING_CONTROL");
    let unicode = error("🙂");
    assert_eq!(unicode.span(), 0..4);
}

#[test]
fn parser_and_resolution_errors_are_fail_closed() {
    for (source, code) in [
        ("", "EXPRESSION_EXPECTED_VALUE"),
        ("1 2", "EXPRESSION_TRAILING_TOKEN"),
        ("(1", "EXPRESSION_EXPECTED_TOKEN"),
        ("min(1", "EXPRESSION_EXPECTED_TOKEN"),
        ("min(1,)", "EXPRESSION_EXPECTED_VALUE"),
        ("watts", "EXPRESSION_UNKNOWN_SYMBOL"),
        ("1fortnight", "EXPRESSION_UNIT"),
        ("mystery(1)", "EXPRESSION_UNKNOWN_FUNCTION"),
        ("min(1)", "EXPRESSION_ARITY"),
        ("clamp(1, 2)", "EXPRESSION_ARITY"),
    ] {
        assert_eq!(error(source).code(), code, "{source}");
    }
}

#[test]
fn arithmetic_errors_do_not_wrap_or_approximate() {
    assert_eq!(error("1 / 0").code(), "EXPRESSION_DIVIDE_BY_ZERO");
    assert_eq!(
        error("170141183460469231731687303715884105727 * 2").code(),
        "EXPRESSION_OVERFLOW"
    );
    assert_eq!(
        error("170141183460469231731687303715884105728").code(),
        "EXPRESSION_NUMBER_LITERAL"
    );
    let tiny = format!("0.{}1ms", "0".repeat(37));
    assert_eq!(error(&tiny).code(), "EXPRESSION_OVERFLOW");
    let tiny_percent = format!("1 / 0.{}1%", "0".repeat(37));
    assert_eq!(error(&tiny_percent).code(), "EXPRESSION_OVERFLOW");

    let mut environment = Environment::new();
    environment.insert(
        "minimum".to_owned(),
        crate::program::expression::Value::Scalar(
            crate::program::expression::ExactNumber::integer(i128::MIN),
        ),
    );
    assert_eq!(
        evaluate("-minimum", &environment).unwrap_err().code(),
        "EXPRESSION_OVERFLOW"
    );
}

#[test]
fn syntax_and_evaluation_budgets_are_enforced() {
    let nested = format!("{}1{}", "(".repeat(65), ")".repeat(65));
    assert_eq!(error(&nested).code(), "EXPRESSION_DEPTH_LIMIT");

    let deep = vec!["1"; 70].join(" + ");
    assert_eq!(error(&deep).code(), "EXPRESSION_DEPTH_LIMIT");

    let large = vec!["1"; 600].join(" + ");
    assert_eq!(error(&large).code(), "EXPRESSION_NODE_LIMIT");

    use crate::program::expression::ast::{Expression, ExpressionKind};
    let literal = Expression {
        kind: ExpressionKind::Literal(crate::program::expression::Value::Scalar(
            crate::program::expression::ExactNumber::integer(1),
        )),
        span: 0..1,
    };
    let oversized = Expression {
        kind: ExpressionKind::Call {
            function: "max".to_owned(),
            arguments: vec![literal; crate::program::expression::MAX_EXPRESSION_NODES],
        },
        span: 0..1,
    };
    assert_eq!(
        crate::program::expression::runtime::evaluate(&oversized, &Environment::new())
            .unwrap_err()
            .code(),
        "EXPRESSION_NODE_LIMIT"
    );
}

#[test]
fn public_evaluate_accepts_an_empty_environment() {
    assert_eq!(
        evaluate("max(1, 2)", &Environment::new()).unwrap().render(),
        "2"
    );
}
