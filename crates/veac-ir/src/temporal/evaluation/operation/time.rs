use crate::{RationalTime, TemporalEvaluationError, TemporalValue};

use super::support::{error, integer, number, time};

pub(super) fn add(
    left: RationalTime,
    right: RationalTime,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    combine(left, right, false, pointer)
}

pub(super) fn subtract(
    left: RationalTime,
    right: RationalTime,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    combine(left, right, true, pointer)
}

pub(super) fn scale(
    value: RationalTime,
    factor: f64,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let scaled = number(value.value as f64 * factor, pointer)?;
    if scaled.fract() != 0.0 {
        return Err(error(
            pointer,
            "scaled time is not exactly representable at its timescale",
        ));
    }
    Ok(TemporalValue::Time {
        value: time(scaled as i128, value.timescale, pointer)?,
    })
}

pub(super) fn divide(
    left: RationalTime,
    right: RationalTime,
    pointer: &str,
) -> Result<f64, TemporalEvaluationError> {
    if right.value == 0 {
        return Err(error(pointer, "division by zero"));
    }
    let numerator = i128::from(left.value) * i128::from(right.timescale);
    let denominator = i128::from(right.value) * i128::from(left.timescale);
    number(numerator as f64 / denominator as f64, pointer)
}

pub(super) fn ratio(
    value: RationalTime,
    left: RationalTime,
    right: RationalTime,
    pointer: &str,
) -> Result<f64, TemporalEvaluationError> {
    let numerator = delta(value, left, right.timescale, pointer)?;
    let denominator = delta(right, left, value.timescale, pointer)?;
    if denominator == 0 {
        return Err(error(pointer, "time interpolation interval is empty"));
    }
    number(numerator as f64 / denominator as f64, pointer)
}

fn combine(
    left: RationalTime,
    right: RationalTime,
    subtract: bool,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    if left.timescale == right.timescale {
        let right = if subtract {
            -i128::from(right.value)
        } else {
            i128::from(right.value)
        };
        return Ok(TemporalValue::Time {
            value: time(i128::from(left.value) + right, left.timescale, pointer)?,
        });
    }
    let scale_gcd = gcd(u128::from(left.timescale), u128::from(right.timescale));
    let left_factor = u128::from(right.timescale) / scale_gcd;
    let right_factor = u128::from(left.timescale) / scale_gcd;
    let left_value = i128::from(left.value) * as_i128(left_factor, pointer)?;
    let right_value = i128::from(right.value) * as_i128(right_factor, pointer)?;
    let numerator = if subtract {
        left_value - right_value
    } else {
        left_value + right_value
    };
    let denominator = match right_factor.checked_mul(u128::from(right.timescale)) {
        Some(value) => value,
        None => return Err(error(pointer, "time denominator overflowed")),
    };
    reduced(numerator, denominator, pointer)
}

fn reduced(
    numerator: i128,
    denominator: u128,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let divisor = gcd(numerator.unsigned_abs(), denominator);
    let numerator = numerator / as_i128(divisor, pointer)?;
    let denominator = denominator / divisor;
    let timescale = match u32::try_from(denominator) {
        Ok(value) => value,
        Err(_) => {
            return Err(error(
                pointer,
                "exact time denominator exceeds the supported range",
            ))
        }
    };
    let value = match RationalTime::new(integer(numerator, pointer)?, timescale) {
        Ok(value) => value,
        Err(cause) => return Err(error(pointer, &cause.to_string())),
    };
    Ok(TemporalValue::Time { value })
}

fn delta(
    sample: RationalTime,
    origin: RationalTime,
    multiplier: u32,
    pointer: &str,
) -> Result<i128, TemporalEvaluationError> {
    let Some(scaled) = i128::from(sample.value).checked_mul(i128::from(origin.timescale)) else {
        return Err(error(pointer, "time interpolation arithmetic overflowed"));
    };
    let Some(delta) = scaled.checked_sub(i128::from(origin.value) * i128::from(sample.timescale))
    else {
        return Err(error(pointer, "time interpolation arithmetic overflowed"));
    };
    delta
        .checked_mul(i128::from(multiplier))
        .ok_or(error(pointer, "time interpolation arithmetic overflowed"))
}

fn as_i128(value: u128, pointer: &str) -> Result<i128, TemporalEvaluationError> {
    match i128::try_from(value) {
        Ok(value) => Ok(value),
        Err(_) => Err(error(pointer, "time arithmetic overflowed")),
    }
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}
