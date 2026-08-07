use super::closure::VerifiedClosureDefinition;
use super::error;
use crate::program::expression::core::{
    ClosureDefinitionId, CoreInstructionKind, CoreProgram, CoreTerminator, ValueId,
};
use crate::program::expression::ExpressionError;

pub(super) fn verify(
    program: &CoreProgram,
    closures: &[std::sync::Arc<VerifiedClosureDefinition>],
) -> Result<(), ExpressionError> {
    for definition in closures.iter().filter(|value| value.non_escaping()) {
        let id = definition.id();
        let materialized = materializations(program, id);
        let owners = for_each_owners(program, id);
        match (materialized.as_slice(), owners.as_slice()) {
            ([(value, span)], []) => verify_uses(program, *value, span.clone())?,
            ([], [_]) => {}
            _ => {
                return Err(error(
                    "non-escaping closure definition must have exactly one Collection or for-each owner",
                    definition_span(program, id),
                ));
            }
        }
    }
    Ok(())
}

fn materializations(
    program: &CoreProgram,
    definition: ClosureDefinitionId,
) -> Vec<(ValueId, std::ops::Range<usize>)> {
    program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .filter_map(|value| match &value.kind {
            CoreInstructionKind::Closure { definition: id, .. } if *id == definition => {
                Some((value.id, value.span.clone()))
            }
            _ => None,
        })
        .collect()
}

fn for_each_owners(
    program: &CoreProgram,
    definition: ClosureDefinitionId,
) -> Vec<std::ops::Range<usize>> {
    program
        .blocks
        .iter()
        .filter_map(|block| match &block.terminator {
            CoreTerminator::ForEach(value) if value.body() == definition => {
                Some(value.provenance().loop_span().clone())
            }
            _ => None,
        })
        .collect()
}

fn definition_span(
    program: &CoreProgram,
    definition: ClosureDefinitionId,
) -> std::ops::Range<usize> {
    definition
        .index()
        .and_then(|index| program.closure_definitions.get(index))
        .map(|value| value.span())
        .unwrap_or(0..0)
}

fn verify_uses(
    program: &CoreProgram,
    closure: ValueId,
    closure_span: std::ops::Range<usize>,
) -> Result<(), ExpressionError> {
    let mut collection_uses = 0usize;
    for block in &program.blocks {
        for instruction in &block.instructions {
            let occurrences = instruction
                .kind
                .operands()
                .filter(|operand| *operand == closure)
                .count();
            if occurrences == 0 {
                continue;
            }
            let allowed = matches!(
                instruction.kind,
                CoreInstructionKind::Collection { callable, .. } if callable == closure
            ) && occurrences == 1;
            if !allowed {
                return Err(error(
                    "non-escaping closure may only be a Collection callback",
                    instruction.span.clone(),
                ));
            }
            collection_uses += 1;
        }
        if terminator_uses(&block.terminator, closure) {
            return Err(error(
                "non-escaping closure cannot flow through control or Return",
                terminator_span(&block.terminator),
            ));
        }
    }
    (collection_uses == 1).then_some(()).ok_or_else(|| {
        error(
            "non-escaping closure must have exactly one Collection callback use",
            closure_span,
        )
    })
}

fn terminator_uses(terminator: &CoreTerminator, value: ValueId) -> bool {
    terminator.operands().any(|operand| operand == value)
}

fn terminator_span(terminator: &CoreTerminator) -> std::ops::Range<usize> {
    terminator.span().clone()
}
