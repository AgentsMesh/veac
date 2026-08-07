use std::ops::Range;

use super::super::super::{ExpressionError, PrimitiveType, ValueType};
use super::type_error;

pub(super) fn same_numeric(
    left: &ValueType,
    right: &ValueType,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    if left.is_numeric() && left == right {
        Ok(left.clone())
    } else {
        Err(type_error(
            format!("cannot combine {left} and {right}"),
            span,
        ))
    }
}

pub(super) fn multiply(
    left: &ValueType,
    right: &ValueType,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    require_numeric(left, right, span.clone())?;
    let (left, right) = primitive_pair(left, right, span.clone())?;
    if left == PrimitiveType::Integer || right == PrimitiveType::Integer {
        return exact_integer_pair(left, right)
            .then_some(PrimitiveType::Integer.into())
            .ok_or_else(|| type_error("int arithmetic requires two int operands", span));
    }
    match (left, right) {
        (PrimitiveType::Scalar, kind) | (kind, PrimitiveType::Scalar) => Ok(kind.into()),
        (PrimitiveType::Percent, PrimitiveType::Percent) => Ok(PrimitiveType::Percent.into()),
        (PrimitiveType::Percent, kind) | (kind, PrimitiveType::Percent) => Ok(kind.into()),
        _ => Err(type_error(
            "multiplication requires a scalar or percent",
            span,
        )),
    }
}

pub(super) fn divide(
    left: &ValueType,
    right: &ValueType,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    require_numeric(left, right, span.clone())?;
    let (left, right) = primitive_pair(left, right, span.clone())?;
    if left == PrimitiveType::Integer || right == PrimitiveType::Integer {
        return exact_integer_pair(left, right)
            .then_some(PrimitiveType::Scalar.into())
            .ok_or_else(|| type_error("int arithmetic requires two int operands", span));
    }
    if left == right {
        Ok(PrimitiveType::Scalar.into())
    } else if right == PrimitiveType::Scalar || right == PrimitiveType::Percent {
        Ok(left.into())
    } else {
        Err(type_error(format!("cannot divide {left} by {right}"), span))
    }
}

fn exact_integer_pair(left: PrimitiveType, right: PrimitiveType) -> bool {
    left == PrimitiveType::Integer && right == PrimitiveType::Integer
}

fn require_numeric(
    left: &ValueType,
    right: &ValueType,
    span: Range<usize>,
) -> Result<(), ExpressionError> {
    if left.is_numeric() && right.is_numeric() {
        Ok(())
    } else {
        Err(type_error(
            format!("numeric operator does not accept {left} and {right}"),
            span,
        ))
    }
}

fn primitive_pair(
    left: &ValueType,
    right: &ValueType,
    span: Range<usize>,
) -> Result<(PrimitiveType, PrimitiveType), ExpressionError> {
    left.as_primitive()
        .zip(right.as_primitive())
        .ok_or_else(|| {
            type_error(
                format!("numeric operator does not accept {left} and {right}"),
                span,
            )
        })
}
