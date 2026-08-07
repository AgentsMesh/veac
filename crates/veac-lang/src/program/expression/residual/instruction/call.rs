use super::super::evaluator::Evaluator;
use super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{CoreCallTarget, CoreInstruction, ValueId};

mod builtin;
mod user;

impl Evaluator<'_> {
    pub(super) fn call(
        &mut self,
        target: CoreCallTarget,
        arguments: &[ValueId],
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let arguments = arguments
            .iter()
            .map(|value| self.value(slots, *value, instruction.span()))
            .collect::<Result<Vec<_>, _>>()?;
        match target {
            CoreCallTarget::Builtin(function) => {
                self.builtin(function, arguments, instruction.span())
            }
            CoreCallTarget::User(id) => self.user(id, arguments, instruction.span()),
        }
    }
}
