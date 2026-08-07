use std::collections::BTreeSet;

use super::super::{
    BlockId, CoreInstruction, CoreInstructionKind, CoreProgram, CoreTerminator, Effect,
};
use super::ResidualizationError;

pub(super) fn instruction(value: &CoreInstruction) -> Result<(), ResidualizationError> {
    if value.metadata().effect() == Effect::Pure
        && !value.metadata().contains_local_mutation()
        && !matches!(value.kind(), CoreInstructionKind::GraphEmit { .. })
    {
        return Ok(());
    }
    Err(ResidualizationError::new(
        "RESIDUAL_EFFECT_UNSUPPORTED",
        "residualization only executes Pure Core instructions",
        value.span(),
    ))
}

pub(super) fn branches_are_pure(program: &CoreProgram, starts: [BlockId; 2]) -> bool {
    let mut pending = Vec::from(starts);
    let mut visited = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let Some(block) = id.index().and_then(|index| program.blocks().get(index)) else {
            return false;
        };
        if block
            .instructions()
            .iter()
            .any(|value| instruction(value).is_err())
        {
            return false;
        }
        match block.terminator() {
            CoreTerminator::Return { .. } => {}
            CoreTerminator::Jump { target, .. } => pending.push(*target),
            CoreTerminator::Branch {
                then_target,
                else_target,
                ..
            } => pending.extend([*then_target, *else_target]),
            CoreTerminator::Match { arms, .. } => {
                pending.extend(arms.iter().map(|arm| arm.target()));
            }
            CoreTerminator::ForEach(value) => {
                if value.effect().summary() != Effect::Pure
                    || value.effect().contains_local_mutation()
                {
                    return false;
                }
                pending.push(value.continuation());
            }
        }
    }
    true
}
