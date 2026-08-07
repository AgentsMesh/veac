use veac_plan::canonical::{TemporalCurveKey, TemporalCurvePosition};

use super::budget::Budget;
use super::composite;
use super::easing;
use super::error::TemporalBackendError;
use super::expression;
use super::value::{number, CompiledValue as Value, Expression, LengthExpression};

pub(super) fn compile(
    input: &Value,
    keys: &[TemporalCurveKey],
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let driver = input
        .numeric()
        .ok_or_else(|| budget.contract("curve driver is not numeric"))?;
    let Some(first) = keys.first() else {
        return Err(budget.contract("curve has no keys"));
    };
    let mut result = Value::literal(&keys.last().expect("first key exists").value, budget)?;
    for pair in keys.windows(2).rev() {
        let left = Value::literal(&pair[0].value, budget)?;
        let sampled = if matches!(
            pair[0].interpolation,
            veac_plan::canonical::Interpolation::Hold
        ) {
            left.clone()
        } else {
            let progress = progress(driver, pair[0].position, pair[1].position, budget)?;
            let amount = easing::compile(&progress, &pair[0].interpolation, budget)?;
            let right = Value::literal(&pair[1].value, budget)?;
            interpolate(&left, &right, &amount, budget)?
        };
        let boundary = position(pair[1].position, budget)?;
        let condition = expression::call2(budget, "lt", driver, &boundary)?;
        result = composite::select(&Value::Boolean(condition), &sampled, &result, budget)?;
    }
    let boundary = position(first.position, budget)?;
    let condition = expression::call2(budget, "lte", driver, &boundary)?;
    let first = Value::literal(&first.value, budget)?;
    composite::select(&Value::Boolean(condition), &first, &result, budget)
}

fn progress(
    input: &Expression,
    left: TemporalCurvePosition,
    right: TemporalCurvePosition,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let left = position(left, budget)?;
    let right = position(right, budget)?;
    let numerator = expression::infix(budget, input, "-", &left)?;
    let denominator = expression::infix(budget, &right, "-", &left)?;
    expression::infix(budget, &numerator, "/", &denominator)
}

fn position(
    value: TemporalCurvePosition,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    match value {
        TemporalCurvePosition::Scalar { value } => number(budget, value),
        TemporalCurvePosition::Time { value } => {
            number(budget, value.value as f64 / f64::from(value.timescale))
        }
    }
}

fn interpolate(
    left: &Value,
    right: &Value,
    amount: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    use Value::*;
    Ok(match (left, right) {
        (Scalar(a), Scalar(b)) => Scalar(lerp(a, b, amount, budget)?),
        (Length(a), Length(b)) => Length(length_lerp(a, b, amount, budget)?),
        (Angle(a), Angle(b)) => Angle(lerp(a, b, amount, budget)?),
        (Vec2(ax, ay), Vec2(bx, by)) => {
            Vec2(lerp(ax, bx, amount, budget)?, lerp(ay, by, amount, budget)?)
        }
        (Point(ax, ay), Point(bx, by)) => Point(
            length_lerp(ax, bx, amount, budget)?,
            length_lerp(ay, by, amount, budget)?,
        ),
        (Rect(a), Rect(b)) => Rect(lerp_four(a, b, amount, false, budget)?),
        (Color(a), Color(b)) => Color(lerp_four(a, b, amount, true, budget)?),
        _ => return Err(budget.contract("curve values are not interpolatable")),
    })
}

fn lerp(
    left: &Expression,
    right: &Expression,
    amount: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let delta = expression::infix(budget, right, "-", left)?;
    let scaled = expression::infix(budget, &delta, "*", amount)?;
    expression::infix(budget, left, "+", &scaled)
}

fn length_lerp(
    left: &LengthExpression,
    right: &LengthExpression,
    amount: &Expression,
    budget: &mut Budget<'_>,
) -> Result<LengthExpression, TemporalBackendError> {
    if left.kind != right.kind {
        return Err(budget.contract("curve length units are incompatible"));
    }
    Ok(LengthExpression {
        value: lerp(&left.value, &right.value, amount, budget)?,
        kind: left.kind,
    })
}

fn lerp_four(
    left: &[Expression; 4],
    right: &[Expression; 4],
    amount: &Expression,
    channels: bool,
    budget: &mut Budget<'_>,
) -> Result<[Expression; 4], TemporalBackendError> {
    let mut output = Vec::with_capacity(4);
    for (left, right) in left.iter().zip(right) {
        let value = lerp(left, right, amount, budget)?;
        output.push(if channels {
            color_channel(&value, budget)?
        } else {
            value
        });
    }
    Ok(output.try_into().expect("four components"))
}

fn color_channel(
    value: &Expression,
    budget: &mut Budget<'_>,
) -> Result<Expression, TemporalBackendError> {
    let rounded = super::operation::unary(
        veac_plan::canonical::TemporalUnaryOperation::Round,
        &Value::Scalar(value.clone()),
        budget,
    )?;
    let Value::Scalar(rounded) = rounded else {
        unreachable!()
    };
    let zero = number(budget, 0.0)?;
    let max = number(budget, 255.0)?;
    let lower = expression::call2(budget, "max", &rounded, &zero)?;
    expression::call2(budget, "min", &lower, &max)
}
