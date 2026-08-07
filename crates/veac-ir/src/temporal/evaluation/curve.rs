use std::cmp::Ordering;

use crate::{
    Color, Interpolation, Point, Rect, TemporalCurveKey, TemporalCurvePosition,
    TemporalEvaluationError, TemporalValue, Vec2,
};

use super::operation::{interpolate_length, number, time_ratio};

pub(super) fn sample(
    input: &TemporalValue,
    keys: &[TemporalCurveKey],
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let first = keys.first().ok_or(contract(pointer))?;
    if compare(input, first.position, pointer)?.is_le() {
        return Ok(first.value.clone());
    }
    for pair in keys.windows(2) {
        if compare(input, pair[1].position, pointer)?.is_lt() {
            if matches!(pair[0].interpolation, Interpolation::Hold) {
                return Ok(pair[0].value.clone());
            }
            let progress = ratio(input, pair[0].position, pair[1].position, pointer)?;
            let amount = number(pair[0].interpolation.evaluate(progress), pointer)?;
            return interpolate(&pair[0].value, &pair[1].value, amount, pointer);
        }
    }
    Ok(keys.last().ok_or(contract(pointer))?.value.clone())
}

fn compare(
    input: &TemporalValue,
    position: TemporalCurvePosition,
    pointer: &str,
) -> Result<Ordering, TemporalEvaluationError> {
    match (input, position) {
        (TemporalValue::Scalar { value }, TemporalCurvePosition::Scalar { value: position }) => {
            value.partial_cmp(&position).ok_or(contract(pointer))
        }
        (TemporalValue::Time { value }, TemporalCurvePosition::Time { value: position }) => {
            value.partial_cmp(&position).ok_or(contract(pointer))
        }
        _ => Err(contract(pointer)),
    }
}

fn ratio(
    input: &TemporalValue,
    left: TemporalCurvePosition,
    right: TemporalCurvePosition,
    pointer: &str,
) -> Result<f64, TemporalEvaluationError> {
    let value = match (input, left, right) {
        (
            TemporalValue::Scalar { value },
            TemporalCurvePosition::Scalar { value: left },
            TemporalCurvePosition::Scalar { value: right },
        ) => (*value - left) / (right - left),
        (
            TemporalValue::Time { value },
            TemporalCurvePosition::Time { value: left },
            TemporalCurvePosition::Time { value: right },
        ) => time_ratio(*value, left, right, pointer)?,
        _ => return Err(contract(pointer)),
    };
    number(value, pointer)
}

fn interpolate(
    left: &TemporalValue,
    right: &TemporalValue,
    amount: f64,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match (left, right) {
        (TemporalValue::Scalar { value: a }, TemporalValue::Scalar { value: b }) => {
            Ok(TemporalValue::Scalar {
                value: blend(*a, *b, amount, pointer)?,
            })
        }
        (TemporalValue::Length { value: a }, TemporalValue::Length { value: b }) => {
            Ok(TemporalValue::Length {
                value: interpolate_length(*a, *b, amount, pointer)?,
            })
        }
        (TemporalValue::Angle { degrees: a }, TemporalValue::Angle { degrees: b }) => {
            Ok(TemporalValue::Angle {
                degrees: blend(*a, *b, amount, pointer)?,
            })
        }
        (TemporalValue::Vec2 { value: a }, TemporalValue::Vec2 { value: b }) => {
            Ok(TemporalValue::Vec2 {
                value: Vec2 {
                    x: blend(a.x, b.x, amount, pointer)?,
                    y: blend(a.y, b.y, amount, pointer)?,
                },
            })
        }
        (TemporalValue::Point { value: a }, TemporalValue::Point { value: b }) => {
            Ok(TemporalValue::Point {
                value: Point {
                    x: interpolate_length(a.x, b.x, amount, pointer)?,
                    y: interpolate_length(a.y, b.y, amount, pointer)?,
                },
            })
        }
        (TemporalValue::Rect { value: a }, TemporalValue::Rect { value: b }) => {
            Ok(TemporalValue::Rect {
                value: Rect {
                    x: blend(a.x, b.x, amount, pointer)?,
                    y: blend(a.y, b.y, amount, pointer)?,
                    width: blend(a.width, b.width, amount, pointer)?,
                    height: blend(a.height, b.height, amount, pointer)?,
                },
            })
        }
        (TemporalValue::Color { value: a }, TemporalValue::Color { value: b }) => {
            Ok(TemporalValue::Color {
                value: Color {
                    red: channel(a.red, b.red, amount),
                    green: channel(a.green, b.green, amount),
                    blue: channel(a.blue, b.blue, amount),
                    alpha: channel(a.alpha, b.alpha, amount),
                },
            })
        }
        _ => Err(contract(pointer)),
    }
}

fn blend(
    left: f64,
    right: f64,
    amount: f64,
    pointer: &str,
) -> Result<f64, TemporalEvaluationError> {
    number(left + (right - left) * amount, pointer)
}

fn channel(left: u8, right: u8, amount: f64) -> u8 {
    (f64::from(left) + (f64::from(right) - f64::from(left)) * amount)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn contract(pointer: &str) -> TemporalEvaluationError {
    TemporalEvaluationError::new(
        "TEMPORAL_EVALUATION_CONTRACT",
        pointer,
        "verified curve is incompatible",
    )
}
