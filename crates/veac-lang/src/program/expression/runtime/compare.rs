use std::cmp::Ordering;
use std::ops::Range;

use crate::program::expression::core::{ComparisonOperator, EqualityOperator};
use crate::program::expression::{ExpressionError, Value};

pub(super) fn ordering(
    operator: ComparisonOperator,
    left: Value,
    right: Value,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    let comparison = left
        .numeric()
        .zip(right.numeric())
        .filter(|_| left.primitive_kind() == right.primitive_kind())
        .map(|(left, right)| left.compare(right))
        .ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_TYPE",
                "ordering requires values of one numeric kind",
                span,
            )
        })?;
    let value = match operator {
        ComparisonOperator::Less => comparison == Ordering::Less,
        ComparisonOperator::LessEqual => comparison != Ordering::Greater,
        ComparisonOperator::Greater => comparison == Ordering::Greater,
        ComparisonOperator::GreaterEqual => comparison != Ordering::Less,
    };
    Ok(Value::Bool(value))
}

pub(super) fn equality(
    operator: EqualityOperator,
    left: Value,
    right: Value,
    span: Range<usize>,
) -> Result<Value, ExpressionError> {
    if left.kind() != right.kind() {
        return Err(ExpressionError::new(
            "EXPRESSION_TYPE",
            "equality requires values of one type",
            span,
        ));
    }
    let equal = left == right;
    Ok(Value::Bool(match operator {
        EqualityOperator::Equal => equal,
        EqualityOperator::NotEqual => !equal,
    }))
}
