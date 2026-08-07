mod result;

use crate::{TemporalBinaryOperation as Op, TemporalEvaluationError, TemporalValue};

use super::{
    compare, length,
    support::{contract, error},
    time,
};
use result::{angle, int, length, scalar, vector};

pub(super) fn evaluate(
    operation: Op,
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match operation {
        Op::Add => add(left, right, pointer),
        Op::Subtract => subtract(left, right, pointer),
        Op::Multiply => multiply(left, right, pointer),
        Op::Divide => divide(left, right, pointer),
        Op::Minimum | Op::Maximum => {
            if let (TemporalValue::Length { value: left }, TemporalValue::Length { value: right }) =
                (left, right)
            {
                return length::extreme(*left, *right, matches!(operation, Op::Minimum), pointer);
            }
            let ordering = compare::ordering(left, right, pointer)?;
            let left_wins = matches!(operation, Op::Minimum) == ordering.is_le();
            Ok(if left_wins {
                left.clone()
            } else {
                right.clone()
            })
        }
    }
}

fn add(
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match (left, right) {
        (TemporalValue::Integer { value: a }, TemporalValue::Integer { value: b }) => {
            int(i128::from(*a) + i128::from(*b), pointer)
        }
        (TemporalValue::Scalar { value: a }, TemporalValue::Scalar { value: b }) => {
            scalar(a + b, pointer)
        }
        (TemporalValue::Time { value: a }, TemporalValue::Time { value: b }) => {
            time::add(*a, *b, pointer)
        }
        (TemporalValue::Length { value: a }, TemporalValue::Length { value: b }) => {
            length::add(*a, *b, pointer)
        }
        (TemporalValue::Angle { degrees: a }, TemporalValue::Angle { degrees: b }) => {
            angle(a + b, pointer)
        }
        (TemporalValue::Vec2 { value: a }, TemporalValue::Vec2 { value: b }) => {
            vector(a.x + b.x, a.y + b.y, pointer)
        }
        _ => Err(contract(pointer)),
    }
}

fn subtract(
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match (left, right) {
        (TemporalValue::Integer { value: a }, TemporalValue::Integer { value: b }) => {
            int(i128::from(*a) - i128::from(*b), pointer)
        }
        (TemporalValue::Scalar { value: a }, TemporalValue::Scalar { value: b }) => {
            scalar(a - b, pointer)
        }
        (TemporalValue::Time { value: a }, TemporalValue::Time { value: b }) => {
            time::subtract(*a, *b, pointer)
        }
        (TemporalValue::Length { value: a }, TemporalValue::Length { value: b }) => {
            length::subtract(*a, *b, pointer)
        }
        (TemporalValue::Angle { degrees: a }, TemporalValue::Angle { degrees: b }) => {
            angle(a - b, pointer)
        }
        (TemporalValue::Vec2 { value: a }, TemporalValue::Vec2 { value: b }) => {
            vector(a.x - b.x, a.y - b.y, pointer)
        }
        _ => Err(contract(pointer)),
    }
}

fn multiply(
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match (left, right) {
        (TemporalValue::Integer { value: a }, TemporalValue::Integer { value: b }) => {
            int(i128::from(*a) * i128::from(*b), pointer)
        }
        (TemporalValue::Scalar { value: a }, TemporalValue::Scalar { value: b }) => {
            scalar(a * b, pointer)
        }
        (TemporalValue::Time { value }, TemporalValue::Scalar { value: scale })
        | (TemporalValue::Scalar { value: scale }, TemporalValue::Time { value }) => {
            time::scale(*value, *scale, pointer)
        }
        (TemporalValue::Length { value }, TemporalValue::Scalar { value: scale })
        | (TemporalValue::Scalar { value: scale }, TemporalValue::Length { value }) => {
            length(value.value * scale, value.unit, pointer)
        }
        (TemporalValue::Angle { degrees }, TemporalValue::Scalar { value: scale })
        | (TemporalValue::Scalar { value: scale }, TemporalValue::Angle { degrees }) => {
            angle(degrees * scale, pointer)
        }
        (TemporalValue::Vec2 { value }, TemporalValue::Scalar { value: scale })
        | (TemporalValue::Scalar { value: scale }, TemporalValue::Vec2 { value }) => {
            vector(value.x * scale, value.y * scale, pointer)
        }
        _ => Err(contract(pointer)),
    }
}

fn divide(
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    match (left, right) {
        (_, TemporalValue::Integer { value: 0 }) | (_, TemporalValue::Scalar { value: 0.0 }) => {
            Err(error(pointer, "division by zero"))
        }
        (TemporalValue::Integer { value: a }, TemporalValue::Integer { value: b }) => {
            int(i128::from(*a / *b), pointer)
        }
        (TemporalValue::Scalar { value: a }, TemporalValue::Scalar { value: b }) => {
            scalar(a / b, pointer)
        }
        (TemporalValue::Time { value }, TemporalValue::Scalar { value: scale }) => {
            time::scale(*value, 1.0 / scale, pointer)
        }
        (TemporalValue::Time { value: left }, TemporalValue::Time { value: right }) => {
            scalar(time::divide(*left, *right, pointer)?, pointer)
        }
        (TemporalValue::Length { value }, TemporalValue::Scalar { value: scale }) => {
            length(value.value / scale, value.unit, pointer)
        }
        (TemporalValue::Length { value: left }, TemporalValue::Length { value: right }) => {
            scalar(length::ratio(*left, *right, pointer)?, pointer)
        }
        (TemporalValue::Angle { degrees }, TemporalValue::Scalar { value: scale }) => {
            angle(degrees / scale, pointer)
        }
        (TemporalValue::Angle { degrees: left }, TemporalValue::Angle { degrees: right }) => {
            if *right == 0.0 {
                Err(error(pointer, "division by zero"))
            } else {
                scalar(left / right, pointer)
            }
        }
        (TemporalValue::Vec2 { value }, TemporalValue::Scalar { value: scale }) => {
            vector(value.x / scale, value.y / scale, pointer)
        }
        _ => Err(contract(pointer)),
    }
}
