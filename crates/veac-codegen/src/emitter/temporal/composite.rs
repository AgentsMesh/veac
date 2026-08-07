mod select;

use veac_plan::canonical::{
    TemporalColorChannel, TemporalPointAxis, TemporalRectField, TemporalVectorAxis,
};

use super::budget::Budget;
use super::error::TemporalBackendError;
use super::value::{CompiledValue as Value, Expression};

pub(super) use select::compile as select;

pub(super) fn compose_vec2(
    x: &Value,
    y: &Value,
    budget: &Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    match (x, y) {
        (Value::Scalar(x), Value::Scalar(y)) => Ok(Value::Vec2(x.clone(), y.clone())),
        _ => Err(budget.contract("vector components are incompatible")),
    }
}

pub(super) fn project_vec2(
    value: &Value,
    axis: TemporalVectorAxis,
    budget: &Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let Value::Vec2(x, y) = value else {
        return Err(budget.contract("vector projection operand is incompatible"));
    };
    Ok(Value::Scalar(match axis {
        TemporalVectorAxis::X => x.clone(),
        TemporalVectorAxis::Y => y.clone(),
    }))
}

pub(super) fn compose_point(
    x: &Value,
    y: &Value,
    budget: &Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    match (x, y) {
        (Value::Length(x), Value::Length(y)) => Ok(Value::Point(x.clone(), y.clone())),
        _ => Err(budget.contract("point components are incompatible")),
    }
}

pub(super) fn project_point(
    value: &Value,
    axis: TemporalPointAxis,
    budget: &Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let Value::Point(x, y) = value else {
        return Err(budget.contract("point projection operand is incompatible"));
    };
    Ok(Value::Length(match axis {
        TemporalPointAxis::X => x.clone(),
        TemporalPointAxis::Y => y.clone(),
    }))
}

pub(super) fn compose_rect(
    values: [&Value; 4],
    budget: &Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    Ok(Value::Rect(numeric_four(values, budget)?))
}

pub(super) fn compose_color(
    values: [&Value; 4],
    budget: &Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let expressions = values.map(|value| match value {
        Value::Integer(value) => Ok(value.clone()),
        _ => Err(budget.contract("color channel is incompatible")),
    });
    Ok(Value::Color(
        expressions
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .expect("four channels"),
    ))
}

pub(super) fn project_rect(
    value: &Value,
    field: TemporalRectField,
    budget: &Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let Value::Rect(values) = value else {
        return Err(budget.contract("rect projection operand is incompatible"));
    };
    let index = match field {
        TemporalRectField::X => 0,
        TemporalRectField::Y => 1,
        TemporalRectField::Width => 2,
        TemporalRectField::Height => 3,
    };
    Ok(Value::Scalar(values[index].clone()))
}

pub(super) fn project_color(
    value: &Value,
    channel: TemporalColorChannel,
    budget: &Budget<'_>,
) -> Result<Value, TemporalBackendError> {
    let Value::Color(values) = value else {
        return Err(budget.contract("color projection operand is incompatible"));
    };
    let index = match channel {
        TemporalColorChannel::Red => 0,
        TemporalColorChannel::Green => 1,
        TemporalColorChannel::Blue => 2,
        TemporalColorChannel::Alpha => 3,
    };
    Ok(Value::Integer(values[index].clone()))
}

fn numeric_four(
    values: [&Value; 4],
    budget: &Budget<'_>,
) -> Result<[Expression; 4], TemporalBackendError> {
    let values = values
        .into_iter()
        .map(|value| match value {
            Value::Scalar(value) => Ok(value.clone()),
            _ => Err(budget.contract("rect component is incompatible")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(values.try_into().expect("four components"))
}
