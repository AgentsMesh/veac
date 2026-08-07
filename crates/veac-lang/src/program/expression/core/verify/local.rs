use super::control::ControlFlow;
use super::definitions::Definitions;
use super::{error, type_table};
use crate::program::expression::core::{
    CoreInstruction, CoreInstructionKind, CoreLocalSlot, CoreProgram, CoreValueMetadata,
    LocalSlotId,
};
use crate::program::expression::{ExpressionError, Stage, ValueType};

mod declaration;

pub(super) fn verify(
    program: &CoreProgram,
    _definitions: &Definitions,
    control: &ControlFlow,
    nominal_types: &crate::program::TypeRegistry,
    parameters: &[ValueType],
    parameter_stages: &[Stage],
    captures: &[ValueType],
) -> Result<(), ExpressionError> {
    declaration::verify(
        program,
        nominal_types,
        parameters,
        parameter_stages,
        captures,
    )?;
    let mut initializers = vec![None; program.local_slots.len()];
    for (block, value) in program.blocks.iter().enumerate() {
        for (position, instruction) in value.instructions.iter().enumerate() {
            let Some((slot, initialize)) = local_access(instruction) else {
                continue;
            };
            let index = slot
                .index()
                .filter(|index| *index < initializers.len())
                .ok_or_else(|| error("unknown local slot", instruction.span()))?;
            if initialize && initializers[index].replace((block, position)).is_some() {
                return Err(error(
                    "local slot must be initialized exactly once",
                    instruction.span(),
                ));
            }
        }
    }
    if initializers.iter().any(Option::is_none) {
        return Err(error("every local slot must have an initializer", 0..0));
    }
    verify_dominance(program, control, &initializers)
}

fn verify_dominance(
    program: &CoreProgram,
    control: &ControlFlow,
    initializers: &[Option<(usize, usize)>],
) -> Result<(), ExpressionError> {
    for (block, value) in program.blocks.iter().enumerate() {
        for (position, instruction) in value.instructions.iter().enumerate() {
            let Some((slot, initialize)) = local_access(instruction) else {
                continue;
            };
            if initialize {
                continue;
            }
            let (owner, defined) = initializers[slot.index().expect("verified local slot")]
                .expect("verified initializer");
            let dominated = if owner == block {
                defined < position
            } else {
                control.dominates(owner, block)
            };
            if !dominated {
                return Err(error(
                    "local slot initializer does not dominate its access",
                    instruction.span(),
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn instruction_type(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<ValueType, ExpressionError> {
    let slot = slot(program, instruction)?;
    let expected = type_table::value(program, slot.type_id, instruction.span())?.clone();
    if let CoreInstructionKind::LocalInit { value, .. }
    | CoreInstructionKind::LocalSet { value, .. } = &instruction.kind
    {
        let actual = type_table::value(program, definitions.type_id(*value), instruction.span())?;
        if actual != &expected {
            return Err(error(
                "local assignment type does not match its slot",
                instruction.span(),
            ));
        }
    }
    Ok(expected)
}

pub(super) fn instruction_metadata(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<CoreValueMetadata, ExpressionError> {
    let slot = slot(program, instruction)?;
    match &instruction.kind {
        CoreInstructionKind::LocalInit { value, .. }
        | CoreInstructionKind::LocalSet { value, .. } => Ok(CoreValueMetadata::local_mutation([
            &slot.metadata,
            definitions.metadata(*value),
        ])),
        CoreInstructionKind::LocalGet { .. } => Ok(slot.metadata.clone()),
        _ => unreachable!("called only for local instructions"),
    }
}

fn slot<'a>(
    program: &'a CoreProgram,
    instruction: &CoreInstruction,
) -> Result<&'a CoreLocalSlot, ExpressionError> {
    let (slot, _) = local_access(instruction)
        .ok_or_else(|| error("instruction is not a local access", instruction.span()))?;
    slot.index()
        .and_then(|index| program.local_slots.get(index))
        .filter(|value| value.id == slot)
        .ok_or_else(|| error("unknown local slot", instruction.span()))
}

fn local_access(instruction: &CoreInstruction) -> Option<(LocalSlotId, bool)> {
    match &instruction.kind {
        CoreInstructionKind::LocalInit { slot, .. } => Some((*slot, true)),
        CoreInstructionKind::LocalSet { slot, .. } | CoreInstructionKind::LocalGet { slot } => {
            Some((*slot, false))
        }
        _ => None,
    }
}
