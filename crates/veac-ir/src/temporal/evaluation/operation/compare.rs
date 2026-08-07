use std::cmp::Ordering;

use crate::{TemporalCompareOperation as Op, TemporalEvaluationError, TemporalValue};

use super::{length, support::contract};

pub(super) fn evaluate(
    operation: Op,
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    let value = match operation {
        Op::Equal => equal(left, right, pointer)?,
        Op::NotEqual => !equal(left, right, pointer)?,
        Op::Less => ordering(left, right, pointer)?.is_lt(),
        Op::LessOrEqual => ordering(left, right, pointer)?.is_le(),
        Op::Greater => ordering(left, right, pointer)?.is_gt(),
        Op::GreaterOrEqual => ordering(left, right, pointer)?.is_ge(),
    };
    Ok(TemporalValue::Boolean { value })
}

fn equal(
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<bool, TemporalEvaluationError> {
    Ok(match (left, right) {
        (TemporalValue::Time { .. }, TemporalValue::Time { .. }) => {
            ordering(left, right, pointer)?.is_eq()
        }
        (TemporalValue::Length { value: a }, TemporalValue::Length { value: b }) => {
            length::equal(*a, *b, pointer)?
        }
        (TemporalValue::Boolean { value: a }, TemporalValue::Boolean { value: b }) => a == b,
        (TemporalValue::Integer { value: a }, TemporalValue::Integer { value: b }) => a == b,
        (TemporalValue::Scalar { value: a }, TemporalValue::Scalar { value: b }) => a == b,
        (TemporalValue::Angle { degrees: a }, TemporalValue::Angle { degrees: b }) => a == b,
        (TemporalValue::Vec2 { value: a }, TemporalValue::Vec2 { value: b }) => a == b,
        (TemporalValue::Point { value: a }, TemporalValue::Point { value: b }) => {
            length::equal(a.x, b.x, pointer)? && length::equal(a.y, b.y, pointer)?
        }
        (TemporalValue::Rect { value: a }, TemporalValue::Rect { value: b }) => a == b,
        (TemporalValue::Color { value: a }, TemporalValue::Color { value: b }) => a == b,
        (TemporalValue::Text { value: a }, TemporalValue::Text { value: b }) => a == b,
        _ => return Err(contract(pointer)),
    })
}

pub(super) fn ordering(
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<Ordering, TemporalEvaluationError> {
    match (left, right) {
        (TemporalValue::Integer { value: a }, TemporalValue::Integer { value: b }) => Ok(a.cmp(b)),
        (TemporalValue::Scalar { value: a }, TemporalValue::Scalar { value: b }) => {
            a.partial_cmp(b).ok_or(contract(pointer))
        }
        (TemporalValue::Time { value: a }, TemporalValue::Time { value: b }) => {
            a.partial_cmp(b).ok_or(contract(pointer))
        }
        (TemporalValue::Length { value: a }, TemporalValue::Length { value: b }) => {
            length::ordering(*a, *b, pointer)
        }
        (TemporalValue::Angle { degrees: a }, TemporalValue::Angle { degrees: b }) => {
            a.partial_cmp(b).ok_or(contract(pointer))
        }
        _ => Err(contract(pointer)),
    }
}
