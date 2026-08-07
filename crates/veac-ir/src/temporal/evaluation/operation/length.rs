use std::cmp::Ordering;

use crate::{Length, LengthUnit, TemporalEvaluationError, TemporalValue};

use super::support::{error, number};

pub(super) fn add(
    left: Length,
    right: Length,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    result(left, right, pointer, |a, b| a + b)
}

pub(super) fn subtract(
    left: Length,
    right: Length,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    result(left, right, pointer, |a, b| a - b)
}

pub(super) fn extreme(
    left: Length,
    right: Length,
    minimum: bool,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let (left, right, unit) = common(left, right, pointer)?;
    let value = if minimum == (left <= right) {
        left
    } else {
        right
    };
    value_result(value, unit, pointer)
}

pub(super) fn equal(
    left: Length,
    right: Length,
    pointer: &str,
) -> Result<bool, TemporalEvaluationError> {
    let (left, right, _) = common(left, right, pointer)?;
    Ok(left == right)
}

pub(super) fn ordering(
    left: Length,
    right: Length,
    pointer: &str,
) -> Result<Ordering, TemporalEvaluationError> {
    let (left, right, _) = common(left, right, pointer)?;
    left.partial_cmp(&right)
        .ok_or(error(pointer, "length comparison is not finite"))
}

pub(super) fn interpolate(
    left: Length,
    right: Length,
    amount: f64,
    pointer: &str,
) -> Result<Length, TemporalEvaluationError> {
    let (left, right, unit) = common(left, right, pointer)?;
    Ok(Length {
        value: number(left + (right - left) * amount, pointer)?,
        unit,
    })
}

pub(super) fn ratio(
    left: Length,
    right: Length,
    pointer: &str,
) -> Result<f64, TemporalEvaluationError> {
    let (left, right, _) = common(left, right, pointer)?;
    if right == 0.0 {
        Err(error(pointer, "division by zero"))
    } else {
        number(left / right, pointer)
    }
}

fn result(
    left: Length,
    right: Length,
    pointer: &str,
    operation: impl FnOnce(f64, f64) -> f64,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let (left, right, unit) = common(left, right, pointer)?;
    value_result(operation(left, right), unit, pointer)
}

fn value_result(
    value: f64,
    unit: LengthUnit,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Length {
        value: Length {
            value: number(value, pointer)?,
            unit,
        },
    })
}

fn common(
    left: Length,
    right: Length,
    pointer: &str,
) -> Result<(f64, f64, LengthUnit), TemporalEvaluationError> {
    if left.unit == right.unit {
        return Ok((left.value, right.value, left.unit));
    }
    if relative(left.unit) && relative(right.unit) {
        return Ok((normalized(left), normalized(right), LengthUnit::Normalized));
    }
    Err(error(
        pointer,
        "pixel and relative lengths require an explicit output extent",
    ))
}

fn relative(unit: LengthUnit) -> bool {
    matches!(unit, LengthUnit::Normalized | LengthUnit::Percent)
}

fn normalized(value: Length) -> f64 {
    match value.unit {
        LengthUnit::Normalized => value.value,
        LengthUnit::Percent => value.value / 100.0,
        LengthUnit::Pixels => value.value,
    }
}
