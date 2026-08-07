use super::{error, value};
use crate::program::expression::{evaluate, ExactNumber, Value};

#[test]
fn honors_precedence_parentheses_and_unary_signs() {
    assert_eq!(value("1 + 2 * 3").render(), "7");
    assert_eq!(value("1-2").render(), "-1");
    assert_eq!(value("(1 + 2) * 3").render(), "9");
    assert_eq!(value("--2 + +1").render(), "3");
    assert_eq!(value("-250ms / -2.0").render(), "125ms");
    assert_eq!(value("1 / 3 + 1 / 6").render(), "0.5");
}

#[test]
fn distinguishes_hyphenated_symbols_from_subtraction() {
    let environment = [
        ("title-size", 4),
        ("left", 7),
        ("right", 2),
        ("foo", 1),
        ("bar", 3),
    ]
    .into_iter()
    .map(|(name, number)| (name.to_owned(), Value::Scalar(ExactNumber::integer(number))))
    .collect();

    assert_eq!(
        evaluate("title-size", &environment).unwrap().render(),
        "4.0"
    );
    assert_eq!(
        evaluate("-title-size", &environment).unwrap().render(),
        "-4.0"
    );
    assert_eq!(
        evaluate("left - right", &environment).unwrap().render(),
        "5.0"
    );
    assert_eq!(evaluate("foo--bar", &environment).unwrap().render(), "4.0");
    assert_eq!(evaluate("1-2", &environment).unwrap().render(), "-1");
}

#[test]
fn applies_strict_dimension_arithmetic() {
    assert_eq!(
        value(r#""VEAC" + " language""#).render(),
        r#""VEAC language""#
    );
    assert_eq!(value("1s + 250ms").render(), "1250ms");
    assert_eq!(value("4px - 1.5px").render(), "2.5px");
    assert_eq!(value("2.0 * 3deg").render(), "6deg");
    assert_eq!(value("3deg * 2.0").render(), "6deg");
    assert_eq!(value("80% * 2.0").render(), "160%");
    assert_eq!(value("2.0 * 80%").render(), "160%");
    assert_eq!(value("50% * 50%").render(), "25%");
    assert_eq!(value("200px * 25%").render(), "50px");
    assert_eq!(value("25% * 200px").render(), "50px");
    assert_eq!(value("8s / 2.0").render(), "4s");
    assert_eq!(value("8s / 25%").render(), "32s");
    assert_eq!(value("8s / 2s").render(), "4.0");
    assert_eq!(value("1.0 / 25%").render(), "4.0");
    assert_eq!(value("50% / 2.0").render(), "25%");
}

#[test]
fn evaluates_min_max_and_clamp_exactly() {
    assert_eq!(value("min(2s, 1500ms)").render(), "1500ms");
    assert_eq!(value("max(2s, 1500ms)").render(), "2s");
    assert_eq!(value("min(2s, 2s)").render(), "2s");
    assert_eq!(value("max(2s, 2s)").render(), "2s");
    assert_eq!(value("clamp(3px, 1px, 2px)").render(), "2px");
    assert_eq!(value("clamp(0px, 1px, 2px)").render(), "1px");
    assert_eq!(value("clamp(1.5px, 1px, 2px)").render(), "1.5px");
}

#[test]
fn rejects_invalid_operator_and_function_types() {
    assert_eq!(error("1s + 1px").code(), "EXPRESSION_TYPE");
    assert_eq!(error("1s * 1px").code(), "EXPRESSION_TYPE");
    assert_eq!(error("1s / 1px").code(), "EXPRESSION_TYPE");
    assert_eq!(error(r#"-"x""#).code(), "EXPRESSION_TYPE");
    assert_eq!(error("true + false").code(), "EXPRESSION_TYPE");
    assert_eq!(error("min(1, 1s)").code(), "EXPRESSION_CALL_ARGUMENT_TYPE");
    assert_eq!(
        error("min(true, false)").code(),
        "EXPRESSION_CALL_ARGUMENT_TYPE"
    );
    assert_eq!(error("clamp(1, 3, 2)").code(), "EXPRESSION_CLAMP_RANGE");
}

#[test]
fn text_concatenation_has_a_deterministic_output_budget() {
    let environment = [
        ("left".to_owned(), Value::Text("a".repeat(600_000).into())),
        ("right".to_owned(), Value::Text("b".repeat(600_000).into())),
    ]
    .into_iter()
    .collect();
    assert_eq!(
        evaluate("left + right", &environment).unwrap_err().code(),
        "EXPRESSION_TEXT_LIMIT"
    );
    let literal = format!("\"{}\"", "x".repeat(1024 * 1024 + 1));
    assert_eq!(error(&literal).code(), "EXPRESSION_TEXT_LIMIT");
    let oversized = [(
        "value".to_owned(),
        Value::Text("x".repeat(1024 * 1024 + 1).into()),
    )]
    .into_iter()
    .collect();
    assert_eq!(
        evaluate("value", &oversized).unwrap_err().code(),
        "EXPRESSION_TEXT_LIMIT"
    );
}

#[test]
fn evaluates_comparison_equality_and_boolean_precedence() {
    for (source, expected) in [
        ("1 < 2", true),
        ("2 <= 2", true),
        ("3s > 2500ms", true),
        ("3px >= 4px", false),
        (r#""a" == "a""#, true),
        ("true != false", true),
        ("!false && false || true", true),
    ] {
        assert_eq!(value(source), Value::Bool(expected), "{source}");
    }
    assert_eq!(error("1 < 2s").code(), "EXPRESSION_TYPE");
    assert_eq!(error("1 == 1s").code(), "EXPRESSION_TYPE");
    assert_eq!(error("1 && true").code(), "EXPRESSION_TYPE");
}

#[test]
fn blocks_locals_and_conditionals_execute_lexically() {
    assert_eq!(
        value("{ let base = 2; let score = base * 3; if score > 5 { score } else { 0 } }").render(),
        "6"
    );
    assert_eq!(
        value("{ let value = 1; { let value = 2; value } + value }").render(),
        "3"
    );
    assert_eq!(error("if true { 1 } else { 1s }").code(), "EXPRESSION_TYPE");
    assert_eq!(
        error("{ let value = value; value }").code(),
        "EXPRESSION_UNKNOWN_SYMBOL"
    );
}

#[test]
fn logical_operators_execute_only_the_selected_branch() {
    assert_eq!(value("false && (1 / 0 == 0.0)"), Value::Bool(false));
    assert_eq!(value("true || (1 / 0 == 0.0)"), Value::Bool(true));
    assert_eq!(
        error("true && (1 / 0 == 0.0)").code(),
        "EXPRESSION_DIVIDE_BY_ZERO"
    );
    assert_eq!(
        error("false || (1 / 0 == 0.0)").code(),
        "EXPRESSION_DIVIDE_BY_ZERO"
    );
}
