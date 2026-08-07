use super::super::evaluator::Evaluator;
use super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{CoreInstruction, Value, ValueId};

impl Evaluator<'_> {
    pub(super) fn range(
        &self,
        start: ValueId,
        end: ValueId,
        step: Option<ValueId>,
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let start = integer(self.value(slots, start, instruction.span())?, instruction)?;
        let end = integer(self.value(slots, end, instruction.span())?, instruction)?;
        let step = step
            .map(|id| integer(self.value(slots, id, instruction.span())?, instruction))
            .transpose()?
            .unwrap_or(1);
        Value::range(start, end, step)
            .map(ResidualRuntimeValue::Concrete)
            .map_err(|error| {
                ResidualizationError::new(error.code(), error.message(), instruction.span())
            })
    }
}

fn integer(
    value: ResidualRuntimeValue,
    instruction: &CoreInstruction,
) -> Result<i64, ResidualizationError> {
    match value {
        ResidualRuntimeValue::Concrete(Value::Integer(value)) => Ok(value),
        _ => Err(ResidualizationError::new(
            "RESIDUAL_CORE_CONTRACT",
            "range operand is not a Build-stage integer",
            instruction.span(),
        )),
    }
}
