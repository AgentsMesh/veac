use std::ops::Range;

use super::super::ast::{BinaryOperator, UnaryOperator};
use super::super::{BuiltinFunction, ExpressionError, PrimitiveType, ValueType};

mod arithmetic;
mod builtin;
mod relation;

pub(super) fn unary(
    operator: UnaryOperator,
    value: &ValueType,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    match operator {
        UnaryOperator::Not if value.as_primitive() == Some(PrimitiveType::Boolean) => {
            Ok(PrimitiveType::Boolean.into())
        }
        UnaryOperator::Not => Err(type_error(
            format!("logical negation requires bool, found {value}"),
            span,
        )),
        UnaryOperator::Positive | UnaryOperator::Negative => {
            value.is_numeric().then(|| value.clone()).ok_or_else(|| {
                type_error(
                    format!("unary sign requires a numeric value, found {value}"),
                    span,
                )
            })
        }
    }
}

pub(super) fn binary(
    operator: BinaryOperator,
    left: &ValueType,
    right: &ValueType,
    registry: &crate::program::TypeRegistry,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    match operator {
        BinaryOperator::Add
            if left.as_primitive() == Some(PrimitiveType::Text)
                && right.as_primitive() == Some(PrimitiveType::Text) =>
        {
            Ok(left.clone())
        }
        BinaryOperator::Add | BinaryOperator::Subtract => {
            arithmetic::same_numeric(left, right, span)
        }
        BinaryOperator::Multiply => arithmetic::multiply(left, right, span),
        BinaryOperator::Divide => arithmetic::divide(left, right, span),
        BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual => relation::ordering(left, right, registry, span),
        BinaryOperator::Equal | BinaryOperator::NotEqual => {
            relation::equality(left, right, registry, span)
        }
        BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => {
            relation::logical(left, right, span)
        }
    }
}

pub(super) fn builtin(
    function: BuiltinFunction,
    arguments: &[ValueType],
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    builtin::check(function, arguments, span)
}

pub(super) fn require_arity(
    name: &str,
    actual: usize,
    expected: usize,
    span: Range<usize>,
) -> Result<(), ExpressionError> {
    if actual != expected {
        return Err(ExpressionError::new(
            "EXPRESSION_CALL_ARITY",
            format!("{name} expects {expected} arguments, found {actual}"),
            span,
        ));
    }
    Ok(())
}

fn type_error(message: impl Into<String>, span: Range<usize>) -> ExpressionError {
    ExpressionError::new("EXPRESSION_TYPE", message, span)
}
