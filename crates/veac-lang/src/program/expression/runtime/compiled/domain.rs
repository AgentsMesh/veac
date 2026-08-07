use super::slot::{public, RuntimeValue};
use super::Evaluator;
use crate::program::expression::core::{CoreInstruction, ValueId};
use crate::program::expression::{ExpressionError, Value};

impl Evaluator<'_> {
    pub(super) fn domain_instruction(
        &mut self,
        opcode: u16,
        operands: &[ValueId],
        instruction: &CoreInstruction,
        values: &[Option<RuntimeValue>],
    ) -> Result<Value, ExpressionError> {
        let operands = operands
            .iter()
            .map(|id| public(values, *id, instruction))
            .collect::<Result<Vec<_>, _>>()?;
        let origin = self.origin(instruction.span.clone());
        self.domain
            .evaluate_authored(opcode, operands, instruction.span.clone(), origin)
    }
}
