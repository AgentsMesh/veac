use std::sync::Arc;

use super::apply;
use crate::program::expression::execution_budget::ResourceDelta;
use crate::program::expression::{
    ArithmeticOperator, ExactNumber, ExecutionBudget, Value, MAX_TEXT_VALUE_BYTES,
};

fn text(value: &str) -> Value {
    Value::Text(Arc::from(value))
}

#[test]
fn text_add_reserves_value_bytes_before_building_the_result() {
    let exact = ExecutionBudget::with_limits(usize::MAX, 4);
    assert_eq!(
        apply(
            &exact,
            ArithmeticOperator::Add,
            text("ab"),
            text("cd"),
            3..8,
        )
        .unwrap(),
        text("abcd")
    );

    let short = ExecutionBudget::with_limits(usize::MAX, 3);
    let error = apply(
        &short,
        ArithmeticOperator::Add,
        text("ab"),
        text("cd"),
        3..8,
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(error.span(), 3..8);
    short
        .reserve(
            ResourceDelta {
                value_bytes: 3,
                ..ResourceDelta::default()
            },
            9..10,
        )
        .unwrap();
}

fn number(numerator: i128, denominator: i128) -> ExactNumber {
    ExactNumber::new(numerator, denominator).unwrap()
}

#[test]
fn arithmetic_rejects_invalid_kind_combinations_and_zero_divisors() {
    let budget = ExecutionBudget::default();
    let cases = [
        (
            ArithmeticOperator::Add,
            Value::Integer(1),
            Value::Scalar(number(1, 1)),
        ),
        (ArithmeticOperator::Subtract, text("left"), text("right")),
        (
            ArithmeticOperator::Multiply,
            Value::Integer(1),
            Value::Scalar(number(2, 1)),
        ),
        (
            ArithmeticOperator::Multiply,
            Value::Time(number(1, 1)),
            Value::Time(number(2, 1)),
        ),
        (
            ArithmeticOperator::Divide,
            Value::Integer(1),
            Value::Scalar(number(1, 1)),
        ),
        (
            ArithmeticOperator::Divide,
            Value::Scalar(number(1, 1)),
            Value::Time(number(1, 1)),
        ),
    ];
    for (operator, left, right) in cases {
        assert_eq!(
            apply(&budget, operator, left, right, 4..9)
                .unwrap_err()
                .code(),
            "EXPRESSION_TYPE"
        );
    }
    assert_eq!(
        apply(
            &budget,
            ArithmeticOperator::Divide,
            Value::Integer(1),
            Value::Integer(0),
            7..8
        )
        .unwrap_err()
        .code(),
        "EXPRESSION_DIVIDE_BY_ZERO"
    );
}

#[test]
fn typed_multiplication_and_division_preserve_unit_algebra() {
    let budget = ExecutionBudget::default();
    let percent = Value::Percent(number(50, 1));
    let time = Value::Time(number(8, 1));
    assert_eq!(
        apply(
            &budget,
            ArithmeticOperator::Multiply,
            percent.clone(),
            time.clone(),
            0..1
        )
        .unwrap(),
        Value::Time(number(4, 1))
    );
    assert_eq!(
        apply(
            &budget,
            ArithmeticOperator::Multiply,
            percent.clone(),
            percent,
            0..1
        )
        .unwrap(),
        Value::Percent(number(25, 1))
    );
    assert_eq!(
        apply(
            &budget,
            ArithmeticOperator::Divide,
            time.clone(),
            Value::Scalar(number(2, 1)),
            0..1
        )
        .unwrap(),
        Value::Time(number(4, 1))
    );
    assert_eq!(
        apply(
            &budget,
            ArithmeticOperator::Divide,
            time.clone(),
            Value::Percent(number(25, 1)),
            0..1
        )
        .unwrap(),
        Value::Time(number(32, 1))
    );
    assert_eq!(
        apply(
            &budget,
            ArithmeticOperator::Divide,
            time.clone(),
            time,
            0..1
        )
        .unwrap(),
        Value::Scalar(number(1, 1))
    );
}

#[test]
fn arithmetic_limits_report_stable_codes_and_spans() {
    let budget = ExecutionBudget::default();
    let oversized = "x".repeat(MAX_TEXT_VALUE_BYTES);
    let error = apply(
        &budget,
        ArithmeticOperator::Add,
        text(&oversized),
        text("x"),
        11..17,
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_TEXT_LIMIT");
    assert_eq!(error.span(), 11..17);

    let error = apply(
        &budget,
        ArithmeticOperator::Multiply,
        Value::Integer(i64::MAX),
        Value::Integer(2),
        18..20,
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_OVERFLOW");
    assert_eq!(error.span(), 18..20);
}
