pub(super) mod binary;
pub(super) mod unary;

use veac_plan::canonical::{TemporalBinaryOperation, TemporalUnaryOperation};

use super::{budget::Budget, error::TemporalBackendError, value::CompiledValue};

pub(super) fn unary(
    operation: TemporalUnaryOperation,
    value: &CompiledValue,
    budget: &mut Budget<'_>,
) -> Result<CompiledValue, TemporalBackendError> {
    unary::compile(operation, value, budget)
}

pub(super) fn binary(
    operation: TemporalBinaryOperation,
    left: &CompiledValue,
    right: &CompiledValue,
    budget: &mut Budget<'_>,
) -> Result<CompiledValue, TemporalBackendError> {
    binary::compile(operation, left, right, budget)
}
