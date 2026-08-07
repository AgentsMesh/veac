use super::super::Evaluator;
use crate::program::expression::core::{CoreInstruction, CoreInstructionKind};
use crate::program::expression::{ExpressionError, Value};

impl Evaluator<'_> {
    pub(super) fn account_public(
        &self,
        value: &Value,
        instruction: &CoreInstruction,
    ) -> Result<(), ExpressionError> {
        match &instruction.kind {
            CoreInstructionKind::Input(_)
            | CoreInstructionKind::Parameter(_)
            | CoreInstructionKind::Capture(_)
            | CoreInstructionKind::Closure { .. }
            | CoreInstructionKind::Arithmetic { .. }
            | CoreInstructionKind::StructConstruct { .. }
            | CoreInstructionKind::StructProject { .. }
            | CoreInstructionKind::EnumConstruct { .. } => Ok(()),
            CoreInstructionKind::Literal(_) if value.primitive_kind().is_none() => {
                self.execution.admit_value(value, instruction.span.clone())
            }
            _ => self.execution.charge_value(value, instruction.span.clone()),
        }
    }
}
