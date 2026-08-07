use veac_plan::canonical::TemporalBinaryOperation as Operation;

use super::super::budget::Budget;
use super::super::error::TemporalBackendError;
use super::super::expression;
use super::super::value::{CompiledValue as Value, Expression, LengthExpression};

pub(super) fn compile(
    operation: Operation,
    left: &Value,
    right: &Value,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    use Value::*;
    match (left, right) {
        (Integer(a), Integer(b)) => Ok(Integer(integer(operation, a, b, budget)?)),
        (Scalar(a), Scalar(b)) => Ok(Scalar(numeric(operation, a, b, budget)?)),
        (Time(a), Time(b)) => ratio_or_same(operation, a, b, Value::Time, budget),
        (Length(a), Length(b)) => length_pair(operation, a, b, budget),
        (Angle(a), Angle(b)) => ratio_or_same(operation, a, b, Value::Angle, budget),
        (Vec2(ax, ay), Vec2(bx, by)) => Ok(Vec2(
            numeric(operation, ax, bx, budget)?,
            numeric(operation, ay, by, budget)?,
        )),
        (Time(value), Scalar(scale)) | (Scalar(scale), Time(value)) => {
            scaled(operation, value, scale, Value::Time, budget)
        }
        (Length(value), Scalar(scale)) | (Scalar(scale), Length(value)) => {
            Ok(Length(LengthExpression {
                value: numeric(operation, &value.value, scale, budget)?,
                kind: value.kind,
            }))
        }
        (Angle(value), Scalar(scale)) | (Scalar(scale), Angle(value)) => {
            scaled(operation, value, scale, Value::Angle, budget)
        }
        (Vec2(x, y), Scalar(scale)) | (Scalar(scale), Vec2(x, y)) => Ok(Vec2(
            numeric(operation, x, scale, budget)?,
            numeric(operation, y, scale, budget)?,
        )),
        _ => Err(budget.contract("binary operands are incompatible")),
    }
}

pub(super) fn numeric(
    operation: Operation,
    left: &Expression,
    right: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    match operation {
        Operation::Add => expression::infix(budget, left, "+", right),
        Operation::Subtract => expression::infix(budget, left, "-", right),
        Operation::Multiply => expression::infix(budget, left, "*", right),
        Operation::Divide => expression::infix(budget, left, "/", right),
        Operation::Minimum => expression::call2(budget, "min", left, right),
        Operation::Maximum => expression::call2(budget, "max", left, right),
    }
}

fn integer(
    operation: Operation,
    left: &Expression,
    right: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let value = numeric(operation, left, right, budget)?;
    if operation == Operation::Divide {
        expression::call1(budget, "trunc", &value)
    } else {
        Ok(value)
    }
}

fn ratio_or_same(
    operation: Operation,
    left: &Expression,
    right: &Expression,
    wrap: impl FnOnce(Expression) -> Value,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let output = numeric(operation, left, right, budget)?;
    Ok(if operation == Operation::Divide {
        Value::Scalar(output)
    } else {
        wrap(output)
    })
}

fn scaled(
    operation: Operation,
    value: &Expression,
    scale: &Expression,
    wrap: impl FnOnce(Expression) -> Value,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    Ok(wrap(numeric(operation, value, scale, budget)?))
}

fn length_pair(
    operation: Operation,
    left: &LengthExpression,
    right: &LengthExpression,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    if left.kind != right.kind {
        return Err(
            budget.unsupported("dynamic pixel and relative length arithmetic is unsupported")
        );
    }
    let value = numeric(operation, &left.value, &right.value, budget)?;
    Ok(if operation == Operation::Divide {
        Value::Scalar(value)
    } else {
        Value::Length(LengthExpression {
            value,
            kind: left.kind,
        })
    })
}
