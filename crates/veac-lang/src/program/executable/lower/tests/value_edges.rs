use super::super::value::{color, exact, graph, text};
use crate::program::expression::{ExactNumber, Value};

fn number(value: i128) -> ExactNumber {
    ExactNumber::integer(value)
}

#[test]
fn exact_accepts_every_numeric_runtime_variant() {
    let values = [
        Value::Integer(7),
        Value::Scalar(number(7)),
        Value::Time(number(7)),
        Value::Length(number(7)),
        Value::Angle(number(7)),
    ];
    for value in &values {
        assert_eq!(exact(Some(value)).unwrap(), number(7));
    }
    assert_eq!(
        exact(None).unwrap_err().reason_code(),
        "EXECUTABLE_LOWER_GRAPH"
    );
    assert_eq!(
        exact(Some(&Value::Bool(true))).unwrap_err().reason_code(),
        "EXECUTABLE_LOWER_GRAPH"
    );
}

#[test]
fn text_requires_a_present_text_value() {
    assert_eq!(text(Some(&Value::Text("title".into()))).unwrap(), "title");
    for value in [None, Some(&Value::Integer(1))] {
        assert_eq!(
            text(value).unwrap_err().reason_code(),
            "EXECUTABLE_LOWER_GRAPH"
        );
    }
}

#[test]
fn colors_accept_rgb_and_rgba_in_both_hex_cases() {
    let opaque = color(Some(&Value::Color("#0a10Ff".into()))).unwrap();
    assert_eq!(
        (opaque.red, opaque.green, opaque.blue, opaque.alpha),
        (10, 16, 255, 255)
    );
    let alpha = color(Some(&Value::Color("#A0b1C27f".into()))).unwrap();
    assert_eq!(
        (alpha.red, alpha.green, alpha.blue, alpha.alpha),
        (160, 177, 194, 127)
    );
}

#[test]
fn malformed_colors_fail_with_the_color_contract() {
    let values = [
        Value::Text("#001122".into()),
        Value::Color("001122".into()),
        Value::Color("#00112z".into()),
        Value::Color("#012".into()),
        Value::Color("#0011223344".into()),
    ];
    for value in &values {
        assert_eq!(
            color(Some(value)).unwrap_err().reason_code(),
            "EXECUTABLE_LOWER_COLOR"
        );
    }
}

#[test]
fn graph_errors_keep_the_executable_lowering_code() {
    let error = graph("broken constructor");
    assert_eq!(error.reason_code(), "EXECUTABLE_LOWER_GRAPH");
    assert!(error.to_string().contains("broken constructor"));
}
