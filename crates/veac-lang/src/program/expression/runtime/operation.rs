use std::ops::Range;

use crate::program::expression::ast::BinaryOperator;
use crate::program::expression::{
    ExactNumber, ExpressionError, Value, ValueKind, MAX_TEXT_VALUE_BYTES,
};

pub(super) fn apply(
    operator: BinaryOperator,
    left: Value,
    right: Value,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    match operator {
        BinaryOperator::Add => add(left, right, span),
        BinaryOperator::Subtract => same_kind(left, right, span, ExactNumber::checked_sub),
        BinaryOperator::Multiply => multiply(left, right, span),
        BinaryOperator::Divide => divide(left, right, span),
    }
}

fn add(left: Value, right: Value, span: Range<usize>) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Text(mut left), Value::Text(right)) => {
            let length = left
                .len()
                .checked_add(right.len())
                .ok_or_else(|| overflow(span.clone()))?;
            if length > MAX_TEXT_VALUE_BYTES {
                return Err(ExpressionError::new(
                    "EXPRESSION_TEXT_LIMIT",
                    format!("text value exceeds the {MAX_TEXT_VALUE_BYTES} byte limit"),
                    span,
                ));
            }
            left.push_str(&right);
            Ok(Value::Text(left))
        }
        (left, right) => same_kind(left, right, span, ExactNumber::checked_add),
    }
}

fn same_kind(
    left: Value,
    right: Value,
    span: Range<usize>,
    operation: fn(ExactNumber, ExactNumber) -> Option<ExactNumber>,
) -> Result<Value, ExpressionError> {
    let (left_number, right_number) = numeric_pair(&left, &right, span.clone())?;
    if left.kind() != right.kind() {
        return Err(type_error(
            format!("cannot combine {} and {}", left.kind(), right.kind()),
            span,
        ));
    }
    finish(left.kind(), operation(left_number, right_number), span)
}

fn multiply(left: Value, right: Value, span: Range<usize>) -> Result<Value, ExpressionError> {
    let (left_number, right_number) = numeric_pair(&left, &right, span.clone())?;
    let kind = match (left.kind(), right.kind()) {
        (ValueKind::Scalar, kind) | (kind, ValueKind::Scalar) => kind,
        (ValueKind::Percent, ValueKind::Percent) => ValueKind::Percent,
        (ValueKind::Percent, kind) | (kind, ValueKind::Percent) if kind.is_numeric() => kind,
        _ => {
            return Err(type_error(
                "multiplication requires a scalar or percent",
                span,
            ))
        }
    };
    let mut result = left_number.checked_mul(right_number);
    if left.kind() == ValueKind::Percent && right.kind() != ValueKind::Scalar
        || right.kind() == ValueKind::Percent && left.kind() != ValueKind::Scalar
    {
        result = result.and_then(|value| value.checked_div(ExactNumber::integer(100)));
    }
    finish(kind, result, span)
}

fn divide(left: Value, right: Value, span: Range<usize>) -> Result<Value, ExpressionError> {
    let (left_number, right_number) = numeric_pair(&left, &right, span.clone())?;
    if right_number.is_zero() {
        return Err(ExpressionError::new(
            "EXPRESSION_DIVIDE_BY_ZERO",
            "division by zero",
            span,
        ));
    }
    let (kind, divisor) = if left.kind() == right.kind() {
        (ValueKind::Scalar, right_number)
    } else if right.kind() == ValueKind::Scalar {
        (left.kind(), right_number)
    } else if right.kind() == ValueKind::Percent {
        let ratio = right_number.checked_div(ExactNumber::integer(100));
        (left.kind(), ratio.ok_or_else(|| overflow(span.clone()))?)
    } else {
        return Err(type_error(
            format!("cannot divide {} by {}", left.kind(), right.kind()),
            span,
        ));
    };
    finish(kind, left_number.checked_div(divisor), span)
}

fn numeric_pair(
    left: &Value,
    right: &Value,
    span: Range<usize>,
) -> Result<(ExactNumber, ExactNumber), ExpressionError> {
    left.numeric().zip(right.numeric()).ok_or_else(|| {
        type_error(
            format!(
                "numeric operator does not accept {} and {}",
                left.kind(),
                right.kind()
            ),
            span,
        )
    })
}

fn finish(
    kind: ValueKind,
    value: Option<ExactNumber>,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    value
        .and_then(|value| Value::from_numeric(kind, value))
        .ok_or_else(|| overflow(span))
}

fn type_error(message: impl Into<String>, span: Range<usize>) -> ExpressionError {
    ExpressionError::new("EXPRESSION_TYPE", message, span)
}

fn overflow(span: Range<usize>) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_OVERFLOW",
        "operation exceeds the exact arithmetic range",
        span,
    )
}
