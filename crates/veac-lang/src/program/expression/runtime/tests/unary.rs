use super::super::{unary, validate_value};
use crate::program::expression::core::CoreUnaryOperator;
use crate::program::expression::{ExactNumber, Value, MAX_TEXT_VALUE_BYTES};

#[test]
fn unary_operators_cover_success_type_failure_and_exact_overflow() {
    assert_eq!(
        unary(CoreUnaryOperator::Not, Value::Bool(false), 1..2).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        unary(CoreUnaryOperator::Not, Value::Integer(1), 2..3)
            .unwrap_err()
            .code(),
        "EXPRESSION_TYPE"
    );
    let scalar = Value::Scalar(ExactNumber::integer(2));
    assert_eq!(
        unary(CoreUnaryOperator::Positive, scalar.clone(), 3..4).unwrap(),
        scalar
    );
    assert_eq!(
        unary(CoreUnaryOperator::Positive, Value::Bool(true), 4..5)
            .unwrap_err()
            .code(),
        "EXPRESSION_TYPE"
    );
    assert_eq!(
        unary(CoreUnaryOperator::Negative, Value::Integer(2), 5..6).unwrap(),
        Value::Integer(-2)
    );
    assert_eq!(
        unary(CoreUnaryOperator::Negative, Value::Bool(true), 6..7)
            .unwrap_err()
            .code(),
        "EXPRESSION_TYPE"
    );
    let error = unary(CoreUnaryOperator::Negative, Value::Integer(i64::MIN), 7..9).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_OVERFLOW");
    assert_eq!(error.span(), 7..9);
}

#[test]
fn value_boundary_returns_the_owned_value_only_after_validation() {
    assert_eq!(
        validate_value(Value::Integer(7), 0..1).unwrap(),
        Value::Integer(7)
    );
    let error = validate_value(
        Value::Text("x".repeat(MAX_TEXT_VALUE_BYTES + 1).into()),
        9..12,
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TEXT_LIMIT");
    assert_eq!(error.span(), 9..12);
}
