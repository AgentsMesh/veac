use super::super::error;
use crate::program::expression::core::{CoreClosureDefinition, CoreInstructionKind, CoreProgram};
use crate::program::expression::{ExpressionError, Stage, ValueId};

pub(super) fn verify(
    program: &CoreProgram,
    definition: &CoreClosureDefinition,
) -> Result<(), ExpressionError> {
    let expected = if temporal_definitions(program).contains(&definition.id) {
        Stage::Temporal
    } else {
        Stage::Const
    };
    definition
        .parameter_stages
        .iter()
        .all(|stage| *stage == expected)
        .then_some(())
        .ok_or_else(|| {
            error(
                "closure parameter stages do not match its verified use",
                definition.span(),
            )
        })
}

fn temporal_definitions(
    program: &CoreProgram,
) -> std::collections::BTreeSet<crate::program::expression::ClosureDefinitionId> {
    program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .filter_map(|instruction| match instruction.kind() {
            CoreInstructionKind::TemporalAttach { animation, .. } => {
                closure_definition(program, *animation)
            }
            _ => None,
        })
        .collect()
}

fn closure_definition(
    program: &CoreProgram,
    value: ValueId,
) -> Option<crate::program::expression::ClosureDefinitionId> {
    program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|instruction| instruction.id() == value)
        .and_then(|instruction| match instruction.kind() {
            CoreInstructionKind::Closure { definition, .. } => Some(*definition),
            _ => None,
        })
}
