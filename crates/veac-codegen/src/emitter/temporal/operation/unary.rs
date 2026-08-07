use veac_plan::canonical::TemporalUnaryOperation as Operation;

use super::super::budget::Budget;
use super::super::error::TemporalBackendError;
use super::super::expression;
use super::super::value::{number, CompiledValue as Value, Expression, LengthExpression};

pub(super) fn compile(
    operation: Operation,
    value: &Value,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    use Operation::*;
    if matches!(operation, Negate | Absolute) {
        return mapped(value, budget, |value, budget| match operation {
            Negate => expression::unary(budget, "-(", value, ")"),
            Absolute => expression::call1(budget, "abs", value),
            _ => unreachable!(),
        });
    }
    let input = value
        .numeric()
        .ok_or_else(|| budget.contract("unary operand is not numeric"))?;
    let output = match operation {
        Not => {
            let zero = number(budget, 0.0)?;
            expression::call2(budget, "eq", input, &zero)?
        }
        Floor => expression::call1(budget, "floor", input)?,
        Ceil => expression::call1(budget, "ceil", input)?,
        Round => round(input, budget)?,
        SquareRoot => expression::call1(budget, "sqrt", input)?,
        Sine | Cosine => trig(operation, input, budget)?,
        Exponential => expression::call1(budget, "exp", input)?,
        NaturalLog => expression::call1(budget, "log", input)?,
        Negate | Absolute => unreachable!(),
    };
    Ok(if operation == Not {
        Value::Boolean(output)
    } else {
        Value::Scalar(output)
    })
}

fn mapped(
    value: &Value,
    budget: &mut Budget<'_>,
    mut map: impl FnMut(&Expression, &mut Budget<'_>) -> Result<Expression, TemporalBackendError>,
) -> Result<Value, TemporalBackendError> {
    use Value::*;
    Ok(match value {
        Integer(value) => Integer(map(value, budget)?),
        Scalar(value) => Scalar(map(value, budget)?),
        Time(value) => Time(map(value, budget)?),
        Length(value) => Length(LengthExpression {
            value: map(&value.value, budget)?,
            kind: value.kind,
        }),
        Angle(value) => Angle(map(value, budget)?),
        Vec2(x, y) => Vec2(map(x, budget)?, map(y, budget)?),
        _ => return Err(budget.contract("unary operand is incompatible")),
    })
}

fn round(value: &Expression, budget: &mut Budget<'_>) -> Result<Expression, TemporalBackendError> {
    let zero = number(budget, 0.0)?;
    let half = number(budget, 0.5)?;
    let positive = expression::infix(budget, value, "+", &half)?;
    let negative = expression::infix(budget, value, "-", &half)?;
    let positive = expression::call1(budget, "floor", &positive)?;
    let negative = expression::call1(budget, "ceil", &negative)?;
    let condition = expression::call2(budget, "gte", value, &zero)?;
    expression::select(budget, &condition, &positive, &negative)
}

fn trig(
    operation: Operation,
    value: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let radians = budget.expression(&["(", value, ")*PI/180"])?;
    expression::call1(
        budget,
        if operation == Operation::Sine {
            "sin"
        } else {
            "cos"
        },
        &radians,
    )
}
