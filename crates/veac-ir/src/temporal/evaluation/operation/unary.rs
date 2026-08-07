use crate::{TemporalEvaluationError, TemporalUnaryOperation as Op, TemporalValue, Vec2};

use super::support::{contract, integer, number, time};

pub(super) fn evaluate(
    operation: Op,
    value: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match (operation, value) {
        (Op::Not, TemporalValue::Boolean { value }) => Ok(TemporalValue::Boolean { value: !value }),
        (Op::Negate, TemporalValue::Integer { value }) => Ok(TemporalValue::Integer {
            value: integer(-i128::from(*value), pointer)?,
        }),
        (Op::Negate, TemporalValue::Scalar { value }) => scalar(-value, pointer),
        (Op::Negate, TemporalValue::Time { value }) => Ok(TemporalValue::Time {
            value: time(-i128::from(value.value), value.timescale, pointer)?,
        }),
        (Op::Negate, TemporalValue::Length { value }) => Ok(TemporalValue::Length {
            value: crate::Length {
                value: number(-value.value, pointer)?,
                unit: value.unit,
            },
        }),
        (Op::Negate, TemporalValue::Angle { degrees }) => angle(-degrees, pointer),
        (Op::Negate, TemporalValue::Vec2 { value }) => vector(-value.x, -value.y, pointer),
        (Op::Absolute, TemporalValue::Integer { value }) => Ok(TemporalValue::Integer {
            value: integer(i128::from(*value).abs(), pointer)?,
        }),
        (Op::Absolute, TemporalValue::Scalar { value }) => scalar(value.abs(), pointer),
        (Op::Absolute, TemporalValue::Time { value }) => Ok(TemporalValue::Time {
            value: time(i128::from(value.value).abs(), value.timescale, pointer)?,
        }),
        (Op::Absolute, TemporalValue::Length { value }) => Ok(TemporalValue::Length {
            value: crate::Length {
                value: number(value.value.abs(), pointer)?,
                unit: value.unit,
            },
        }),
        (Op::Absolute, TemporalValue::Angle { degrees }) => angle(degrees.abs(), pointer),
        (Op::Absolute, TemporalValue::Vec2 { value }) => {
            vector(value.x.abs(), value.y.abs(), pointer)
        }
        (Op::Floor, TemporalValue::Scalar { value }) => scalar(value.floor(), pointer),
        (Op::Ceil, TemporalValue::Scalar { value }) => scalar(value.ceil(), pointer),
        (Op::Round, TemporalValue::Scalar { value }) => scalar(value.round(), pointer),
        (Op::SquareRoot, TemporalValue::Scalar { value }) => scalar(value.sqrt(), pointer),
        (Op::Exponential, TemporalValue::Scalar { value }) => scalar(value.exp(), pointer),
        (Op::NaturalLog, TemporalValue::Scalar { value }) => scalar(value.ln(), pointer),
        (Op::Sine, TemporalValue::Angle { degrees }) => scalar(degrees.to_radians().sin(), pointer),
        (Op::Cosine, TemporalValue::Angle { degrees }) => {
            scalar(degrees.to_radians().cos(), pointer)
        }
        _ => Err(contract(pointer)),
    }
}

fn scalar(value: f64, pointer: &str) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Scalar {
        value: number(value, pointer)?,
    })
}

fn angle(value: f64, pointer: &str) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Angle {
        degrees: number(value, pointer)?,
    })
}

fn vector(x: f64, y: f64, pointer: &str) -> Result<TemporalValue, TemporalEvaluationError> {
    Ok(TemporalValue::Vec2 {
        value: Vec2 {
            x: number(x, pointer)?,
            y: number(y, pointer)?,
        },
    })
}
