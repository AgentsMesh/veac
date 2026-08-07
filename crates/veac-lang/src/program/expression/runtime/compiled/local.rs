use super::slot::public_at;
use crate::program::expression::core::{CoreInstruction, CoreInstructionKind};
use crate::program::expression::{ExpressionError, Value};

impl super::Evaluator<'_> {
    pub(super) fn local(
        &self,
        instruction: &CoreInstruction,
        values: &[Option<super::slot::RuntimeValue>],
        locals: &mut [Option<Value>],
    ) -> Result<Value, ExpressionError> {
        match &instruction.kind {
            CoreInstructionKind::LocalInit { slot, value } => {
                let value = public_at(values, *value, &instruction.span)?.clone();
                let target = locals
                    .get_mut(slot.index().expect("verified local slot"))
                    .ok_or_else(|| {
                        contract("local initializer slot is unavailable", instruction)
                    })?;
                if target.replace(value.clone()).is_some() {
                    return Err(contract("local slot was initialized twice", instruction));
                }
                Ok(value)
            }
            CoreInstructionKind::LocalSet { slot, value } => {
                let value = public_at(values, *value, &instruction.span)?.clone();
                let target = locals
                    .get_mut(slot.index().expect("verified local slot"))
                    .ok_or_else(|| contract("local assignment slot is unavailable", instruction))?;
                if target.is_none() {
                    return Err(contract("local slot is not initialized", instruction));
                }
                *target = Some(value.clone());
                Ok(value)
            }
            CoreInstructionKind::LocalGet { slot } => locals
                .get(slot.index().expect("verified local slot"))
                .and_then(Clone::clone)
                .ok_or_else(|| contract("local slot is not initialized", instruction)),
            _ => unreachable!("called only for local instructions"),
        }
    }
}

fn contract(message: &str, instruction: &CoreInstruction) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_RUNTIME_CONTRACT",
        message,
        instruction.span.clone(),
    )
}
