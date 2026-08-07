use std::ops::Range;

use super::error;
use crate::program::expression::core::{ArithmeticOperator, CoreUnaryOperator};
use crate::program::expression::{
    BuiltinFunction, CompiledFunction, ExpressionError, PrimitiveType, ValueType,
};

#[cfg(test)]
mod tests;

pub(super) fn unary(
    operator: CoreUnaryOperator,
    value: &ValueType,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    match operator {
        CoreUnaryOperator::Not if primitive_is(value, PrimitiveType::Boolean) => {
            Ok(primitive(PrimitiveType::Boolean))
        }
        CoreUnaryOperator::Positive | CoreUnaryOperator::Negative if value.is_numeric() => {
            Ok(value.clone())
        }
        _ => Err(error("invalid unary operand type", span)),
    }
}

pub(super) fn arithmetic(
    operator: ArithmeticOperator,
    left: &ValueType,
    right: &ValueType,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    match operator {
        ArithmeticOperator::Add
            if primitive_is(left, PrimitiveType::Text)
                && primitive_is(right, PrimitiveType::Text) =>
        {
            Ok(left.clone())
        }
        ArithmeticOperator::Add | ArithmeticOperator::Subtract
            if left.is_numeric() && left == right =>
        {
            Ok(left.clone())
        }
        ArithmeticOperator::Multiply if left.is_numeric() && right.is_numeric() => {
            multiply(left, right).ok_or_else(|| error("invalid multiplication types", span))
        }
        ArithmeticOperator::Divide if left.is_numeric() && right.is_numeric() => {
            divide(left, right).ok_or_else(|| error("invalid division types", span))
        }
        _ => Err(error("invalid arithmetic operand types", span)),
    }
}

pub(super) fn comparison(
    left: &ValueType,
    right: &ValueType,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    (left.is_numeric() && left == right)
        .then(|| primitive(PrimitiveType::Boolean))
        .ok_or_else(|| error("invalid comparison operand types", span))
}

pub(super) fn equality(
    left: &ValueType,
    right: &ValueType,
    registry: &crate::program::TypeRegistry,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    (left == right && left.contains_function_in(registry) == Some(false))
        .then(|| primitive(PrimitiveType::Boolean))
        .ok_or_else(|| error("invalid equality operand types", span))
}

pub(super) fn builtin(
    function: BuiltinFunction,
    arguments: &[ValueType],
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    let expected = function.arity();
    if arguments.len() != expected {
        return Err(error("builtin call arity mismatch", span));
    }
    function
        .result_type(arguments)
        .map_err(|message| error(message, span))
}

pub(super) fn user_call(
    function: &CompiledFunction,
    arguments: &[ValueType],
    span: Range<usize>,
) -> Result<(), ExpressionError> {
    if arguments.len() != function.parameters().len()
        || arguments
            .iter()
            .zip(function.parameters())
            .any(|(argument, parameter)| argument != &parameter.value_type)
    {
        return Err(error("user call signature mismatch", span));
    }
    Ok(())
}

fn multiply(left: &ValueType, right: &ValueType) -> Option<ValueType> {
    let (left, right) = (left.as_primitive()?, right.as_primitive()?);
    if (left == PrimitiveType::Integer) != (right == PrimitiveType::Integer) {
        return None;
    }
    match (left, right) {
        (PrimitiveType::Integer, PrimitiveType::Integer) => Some(primitive(left)),
        (PrimitiveType::Scalar, kind) | (kind, PrimitiveType::Scalar) => Some(primitive(kind)),
        (PrimitiveType::Percent, PrimitiveType::Percent) => Some(primitive(PrimitiveType::Percent)),
        (PrimitiveType::Percent, kind) | (kind, PrimitiveType::Percent) if kind.is_numeric() => {
            Some(primitive(kind))
        }
        _ => None,
    }
}

fn divide(left: &ValueType, right: &ValueType) -> Option<ValueType> {
    let (left, right) = (left.as_primitive()?, right.as_primitive()?);
    if (left == PrimitiveType::Integer) != (right == PrimitiveType::Integer) {
        return None;
    }
    if left == right {
        Some(primitive(PrimitiveType::Scalar))
    } else if matches!(right, PrimitiveType::Scalar | PrimitiveType::Percent) {
        Some(primitive(left))
    } else {
        None
    }
}

fn primitive(value: PrimitiveType) -> ValueType {
    ValueType::primitive(value)
}

fn primitive_is(value: &ValueType, expected: PrimitiveType) -> bool {
    value.as_primitive() == Some(expected)
}
