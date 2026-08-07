mod binary;
mod compare;
mod composite;
mod length;
mod support;
mod time;
mod unary;

pub(super) use composite::*;

use crate::{
    TemporalBinaryOperation, TemporalCompareOperation, TemporalEvaluationError,
    TemporalUnaryOperation, TemporalValue,
};

pub(super) fn unary(
    operation: TemporalUnaryOperation,
    value: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    unary::evaluate(operation, value, pointer)
}

pub(super) fn binary(
    operation: TemporalBinaryOperation,
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    binary::evaluate(operation, left, right, pointer)
}

pub(super) fn compare(
    operation: TemporalCompareOperation,
    left: &TemporalValue,
    right: &TemporalValue,
    pointer: &str,
) -> Result<TemporalValue, TemporalEvaluationError> {
    compare::evaluate(operation, left, right, pointer)
}

pub(super) fn number(value: f64, pointer: &str) -> Result<f64, TemporalEvaluationError> {
    support::number(value, pointer)
}

pub(super) fn time_ratio(
    value: crate::RationalTime,
    left: crate::RationalTime,
    right: crate::RationalTime,
    pointer: &str,
) -> Result<f64, TemporalEvaluationError> {
    time::ratio(value, left, right, pointer)
}

pub(super) fn interpolate_length(
    left: crate::Length,
    right: crate::Length,
    amount: f64,
    pointer: &str,
) -> Result<crate::Length, TemporalEvaluationError> {
    length::interpolate(left, right, amount, pointer)
}
