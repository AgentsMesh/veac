use crate::{Length, TemporalEvaluationError, TemporalValue, Vec2};

use super::super::support::{integer, number};

pub(super) fn int(value: i128, pointer: &str) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Integer {
        value: integer(value, pointer)?,
    })
}

pub(super) fn scalar(value: f64, pointer: &str) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Scalar {
        value: number(value, pointer)?,
    })
}

pub(super) fn angle(value: f64, pointer: &str) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Angle {
        degrees: number(value, pointer)?,
    })
}

pub(super) fn length(
    value: f64,
    unit: crate::LengthUnit,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Length {
        value: Length {
            value: number(value, pointer)?,
            unit,
        },
    })
}

pub(super) fn vector(
    x: f64,
    y: f64,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Vec2 {
        value: Vec2 {
            x: number(x, pointer)?,
            y: number(y, pointer)?,
        },
    })
}
