use std::ops::Range;

use super::super::super::{ExpressionError, PrimitiveType, ValueType};
use super::type_error;

pub(super) fn ordering(
    left: &ValueType,
    right: &ValueType,
    registry: &crate::program::TypeRegistry,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    if contains_function(left, registry) || contains_function(right, registry) {
        return Err(ExpressionError::new(
            "EXPRESSION_FUNCTION_ORDERING",
            "function values do not support ordering",
            span,
        ));
    }
    if left.is_numeric() && left == right {
        Ok(PrimitiveType::Boolean.into())
    } else {
        Err(type_error(
            format!("ordering requires one numeric kind, found {left} and {right}"),
            span,
        ))
    }
}

pub(super) fn equality(
    left: &ValueType,
    right: &ValueType,
    registry: &crate::program::TypeRegistry,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    if contains_function(left, registry) || contains_function(right, registry) {
        return Err(ExpressionError::new(
            "EXPRESSION_FUNCTION_EQUALITY",
            "function values do not support equality",
            span,
        ));
    }
    (left == right)
        .then_some(PrimitiveType::Boolean.into())
        .ok_or_else(|| type_error(format!("cannot compare {left} and {right}"), span))
}

fn contains_function(value: &ValueType, registry: &crate::program::TypeRegistry) -> bool {
    value.contains_function_in(registry) != Some(false)
}

pub(super) fn logical(
    left: &ValueType,
    right: &ValueType,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    (left.as_primitive() == Some(PrimitiveType::Boolean)
        && right.as_primitive() == Some(PrimitiveType::Boolean))
    .then_some(PrimitiveType::Boolean.into())
    .ok_or_else(|| {
        type_error(
            format!("logical operator requires bool, found {left} and {right}"),
            span,
        )
    })
}
