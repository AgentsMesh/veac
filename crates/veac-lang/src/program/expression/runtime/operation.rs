use std::ops::Range;

use crate::program::expression::core::ArithmeticOperator;
use crate::program::expression::{
    ExactNumber, ExecutionBudget, ExpressionError, PrimitiveType, Value, MAX_TEXT_VALUE_BYTES,
};

pub(super) fn apply(
    execution: &ExecutionBudget,
    operator: ArithmeticOperator,
    left: Value,
    right: Value,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    match operator {
        ArithmeticOperator::Add => add(execution, left, right, span),
        ArithmeticOperator::Subtract => same_kind(left, right, span, ExactNumber::checked_sub),
        ArithmeticOperator::Multiply => multiply(left, right, span),
        ArithmeticOperator::Divide => divide(left, right, span),
    }
}

fn add(
    execution: &ExecutionBudget,
    left: Value,
    right: Value,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    match (left, right) {
        (Value::Text(left), Value::Text(right)) => {
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
            execution.reserve_value_bytes(length, span.clone())?;
            let mut joined = String::with_capacity(length);
            joined.push_str(&left);
            joined.push_str(&right);
            Ok(Value::Text(joined.into()))
        }
        (left, right) => same_kind(left, right, span, ExactNumber::checked_add),
    }
}

#[cfg(test)]
#[path = "operation/tests.rs"]
mod tests;

fn same_kind(
    left: Value,
    right: Value,
    span: Range<usize>,
    operation: fn(ExactNumber, ExactNumber) -> Option<ExactNumber>,
) -> Result<Value, ExpressionError> {
    let (left_number, right_number) = numeric_pair(&left, &right, span.clone())?;
    let left_kind = left.primitive_kind().expect("numeric values are primitive");
    let right_kind = right
        .primitive_kind()
        .expect("numeric values are primitive");
    if left_kind != right_kind {
        return Err(type_error(
            format!("cannot combine {} and {}", left.kind(), right.kind()),
            span,
        ));
    }
    finish(left_kind, operation(left_number, right_number), span)
}

fn multiply(left: Value, right: Value, span: Range<usize>) -> Result<Value, ExpressionError> {
    let (left_number, right_number) = numeric_pair(&left, &right, span.clone())?;
    let left_kind = left.primitive_kind().expect("numeric values are primitive");
    let right_kind = right
        .primitive_kind()
        .expect("numeric values are primitive");
    if (left_kind == PrimitiveType::Integer) != (right_kind == PrimitiveType::Integer) {
        return Err(type_error("int arithmetic requires two int operands", span));
    }
    let kind = match (left_kind, right_kind) {
        (PrimitiveType::Integer, PrimitiveType::Integer) => PrimitiveType::Integer,
        (PrimitiveType::Scalar, kind) | (kind, PrimitiveType::Scalar) => kind,
        (PrimitiveType::Percent, PrimitiveType::Percent) => PrimitiveType::Percent,
        (PrimitiveType::Percent, kind) | (kind, PrimitiveType::Percent) if kind.is_numeric() => {
            kind
        }
        _ => {
            return Err(type_error(
                "multiplication requires a scalar or percent",
                span,
            ))
        }
    };
    let mut result = left_number.checked_mul(right_number);
    if left_kind == PrimitiveType::Percent && right_kind != PrimitiveType::Scalar
        || right_kind == PrimitiveType::Percent && left_kind != PrimitiveType::Scalar
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
    let left_kind = left.primitive_kind().expect("numeric values are primitive");
    let right_kind = right
        .primitive_kind()
        .expect("numeric values are primitive");
    if (left_kind == PrimitiveType::Integer) != (right_kind == PrimitiveType::Integer) {
        return Err(type_error("int arithmetic requires two int operands", span));
    }
    let (kind, divisor) = if left_kind == right_kind {
        (PrimitiveType::Scalar, right_number)
    } else if right_kind == PrimitiveType::Scalar {
        (left_kind, right_number)
    } else if right_kind == PrimitiveType::Percent {
        let ratio = right_number.checked_div(ExactNumber::integer(100));
        (left_kind, ratio.ok_or_else(|| overflow(span.clone()))?)
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
    kind: PrimitiveType,
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
