use veac_plan::canonical::TemporalCompareOperation as Operation;

use super::budget::Budget;
use super::error::TemporalBackendError;
use super::expression;
use super::value::{CompiledValue as Value, Expression, LengthExpression, TextChoice};

pub(super) fn compile(
    operation: Operation,
    left: &Value,
    right: &Value,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let equal_only = matches!(operation, Operation::Equal | Operation::NotEqual);
    let result = match (left, right) {
        (Value::Boolean(a), Value::Boolean(b)) if equal_only => equal(a, b, budget)?,
        (Value::Integer(a), Value::Integer(b))
        | (Value::Scalar(a), Value::Scalar(b))
        | (Value::Time(a), Value::Time(b))
        | (Value::Angle(a), Value::Angle(b)) => ordered(operation, a, b, budget)?,
        (Value::Length(a), Value::Length(b)) => length(operation, a, b, budget)?,
        (Value::Vec2(ax, ay), Value::Vec2(bx, by)) if equal_only => {
            all_equal(&[(ax, bx), (ay, by)], budget)?
        }
        (Value::Point(ax, ay), Value::Point(bx, by)) if equal_only => {
            lengths_equal(&[(ax, bx), (ay, by)], budget)?
        }
        (Value::Rect(a), Value::Rect(b)) | (Value::Color(a), Value::Color(b)) if equal_only => {
            all_equal(&a.iter().zip(b).collect::<Vec<_>>(), budget)?
        }
        (Value::Text(a), Value::Text(b)) if equal_only => text_equal(a, b, budget)?,
        _ => return Err(budget.contract("comparison operands are incompatible")),
    };
    let result = if operation == Operation::NotEqual {
        let zero = super::value::number(budget, 0.0)?;
        expression::call2(budget, "eq", &result, &zero)?
    } else {
        result
    };
    Ok(Value::Boolean(result))
}

fn ordered(
    operation: Operation,
    left: &Expression,
    right: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let function = match operation {
        Operation::Equal | Operation::NotEqual => "eq",
        Operation::Less => "lt",
        Operation::LessOrEqual => "lte",
        Operation::Greater => "gt",
        Operation::GreaterOrEqual => "gte",
    };
    expression::call2(budget, function, left, right)
}

fn equal(
    left: &Expression,
    right: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    expression::call2(budget, "eq", left, right)
}

fn length(
    operation: Operation,
    left: &LengthExpression,
    right: &LengthExpression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    if left.kind != right.kind {
        return Err(budget
            .unsupported("pixel and relative length comparison is unavailable without an extent"));
    }
    ordered(operation, &left.value, &right.value, budget)
}

fn lengths_equal(
    values: &[(&LengthExpression, &LengthExpression)],
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let mut expressions = Vec::with_capacity(values.len());
    for (left, right) in values {
        expressions.push(length(Operation::Equal, left, right, budget)?);
    }
    conjunction(&expressions, budget)
}

fn all_equal(
    values: &[(&Expression, &Expression)],
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let mut expressions = Vec::with_capacity(values.len());
    for (left, right) in values {
        expressions.push(equal(left, right, budget)?);
    }
    conjunction(&expressions, budget)
}

fn conjunction(
    values: &[Expression],
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let Some(first) = values.first() else {
        return super::value::number(budget, 1.0);
    };
    values
        .iter()
        .skip(1)
        .try_fold(first.clone(), |left, right| {
            expression::infix(budget, &left, "*", right)
        })
}

fn text_equal(
    left: &[TextChoice],
    right: &[TextChoice],
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let mut matches = Vec::new();
    for a in left {
        for b in right.iter().filter(|value| value.value == a.value) {
            matches.push(expression::infix(budget, &a.condition, "*", &b.condition)?);
        }
    }
    let zero = super::value::number(budget, 0.0)?;
    let sum = matches.into_iter().try_fold(zero.clone(), |left, right| {
        expression::infix(budget, &left, "+", &right)
    })?;
    expression::call2(budget, "gt", &sum, &zero)
}
