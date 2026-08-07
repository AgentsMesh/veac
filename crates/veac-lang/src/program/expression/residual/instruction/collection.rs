use super::super::evaluator::Evaluator;
use super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{CoreInstruction, Value, ValueId, ValueTypeKind};

mod aggregate;

impl Evaluator<'_> {
    pub(super) fn closure(
        &mut self,
        definition: crate::program::expression::ClosureDefinitionId,
        captures: &[ValueId],
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let reusable = definition
            .index()
            .and_then(|index| self.core().closure_definitions().get(index))
            .is_some()
            && collection_only_use(self.core(), instruction.id());
        if !reusable {
            return Err(ResidualizationError::new(
                "RESIDUAL_INSTRUCTION_UNSUPPORTED",
                "escaping closure values cannot be residualized",
                instruction.span(),
            ));
        }
        let captures = captures
            .iter()
            .map(|id| self.value(slots, *id, instruction.span()))
            .collect::<Result<Vec<_>, _>>()?;
        self.closures.insert(
            (self.call_depth, instruction.id()),
            super::super::closure_state::ResidualClosure {
                definition,
                captures,
            },
        );
        Ok(ResidualRuntimeValue::Concrete(Value::Integer(0)))
    }

    pub(super) fn sequence(
        &self,
        elements: &[ValueId],
        tuple: bool,
        instruction: &CoreInstruction,
        slots: &[Option<ResidualRuntimeValue>],
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        self.concrete_budget
            .reserve_sequence_collection(elements.len(), instruction.span())
            .map_err(ResidualizationError::expression)?;
        let values = elements
            .iter()
            .map(|id| self.value(slots, *id, instruction.span()))
            .collect::<Result<Vec<_>, _>>()?;
        if values
            .iter()
            .all(|value| matches!(value, ResidualRuntimeValue::Concrete(_)))
        {
            return concrete(values, tuple, instruction, self);
        }
        let value_type = self
            .core()
            .value_type(instruction.type_id())
            .cloned()
            .ok_or_else(|| contract("sequence result type is unavailable", instruction))?;
        Ok(ResidualRuntimeValue::Sequence { value_type, values })
    }
}

fn collection_only_use(core: &crate::program::expression::CoreProgram, closure: ValueId) -> bool {
    let mut uses = 0usize;
    for block in core.blocks() {
        for instruction in block.instructions() {
            let occurrences = instruction
                .kind()
                .operands()
                .filter(|operand| *operand == closure)
                .count();
            if occurrences == 0 {
                continue;
            }
            if !matches!(
                instruction.kind(),
                crate::program::expression::CoreInstructionKind::Collection { callable, .. }
                    if *callable == closure && occurrences == 1
            ) {
                return false;
            }
            uses += 1;
        }
        if block.terminator().operands().any(|value| value == closure) {
            return false;
        }
    }
    uses == 1
}

fn concrete(
    values: Vec<ResidualRuntimeValue>,
    tuple: bool,
    instruction: &CoreInstruction,
    evaluator: &Evaluator<'_>,
) -> Result<ResidualRuntimeValue, ResidualizationError> {
    let values = values
        .into_iter()
        .map(|value| match value {
            ResidualRuntimeValue::Concrete(value) => value,
            _ => unreachable!("concrete sequence was checked"),
        })
        .collect();
    let value = if tuple {
        Value::tuple(values)
    } else {
        let value_type = evaluator
            .core()
            .value_type(instruction.type_id())
            .ok_or_else(|| contract("list result type is unavailable", instruction))?;
        let ValueTypeKind::List(element) = value_type.kind() else {
            return Err(contract("list result type is invalid", instruction));
        };
        Value::list(element.clone(), values)
    }
    .map_err(|error| {
        ResidualizationError::new(error.code(), error.message(), instruction.span())
    })?;
    Ok(ResidualRuntimeValue::Concrete(value))
}

fn contract(message: &str, instruction: &CoreInstruction) -> ResidualizationError {
    ResidualizationError::new("RESIDUAL_CORE_CONTRACT", message, instruction.span())
}
