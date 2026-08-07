use super::super::error;
use super::semantic;
use crate::program::expression::core::{CoreProgram, CoreTypeId, CoreTypeTable};
use crate::program::expression::ExpressionError;

pub(super) fn verify(program: &CoreProgram) -> Result<(), ExpressionError> {
    let mut seen = vec![false; program.types.entries().len()];
    let mut next = 0usize;
    for input in &program.inputs {
        visit(program.types(), input.type_id, &mut seen, &mut next)?;
    }
    let mut values = program
        .blocks
        .iter()
        .flat_map(|block| {
            block
                .parameters
                .iter()
                .map(|value| (value.id, value.type_id))
                .chain(
                    block
                        .instructions
                        .iter()
                        .map(|value| (value.id, value.type_id)),
                )
        })
        .collect::<Vec<_>>();
    values.sort_unstable_by_key(|(id, _)| *id);
    let activation = activation_types(program);
    for (value, id) in values {
        visit(program.types(), id, &mut seen, &mut next)?;
        if let Some(types) = activation.get(&value) {
            for id in types {
                visit(program.types(), *id, &mut seen, &mut next)?;
            }
        }
    }
    visit(program.types(), program.result_type, &mut seen, &mut next)?;
    (next == seen.len())
        .then_some(())
        .ok_or_else(|| error("Core type table contains an unused type", 0..0))
}

fn activation_types(
    program: &CoreProgram,
) -> std::collections::BTreeMap<crate::program::expression::ValueId, [CoreTypeId; 2]> {
    program
        .blocks
        .iter()
        .filter_map(|block| match &block.terminator {
            crate::program::expression::CoreTerminator::ForEach(value) => {
                let block = value
                    .continuation()
                    .index()
                    .and_then(|index| program.blocks.get(index))?;
                let parameter = block.parameters.first()?;
                Some((
                    parameter.id(),
                    [value.element().type_id(), value.index().type_id()],
                ))
            }
            _ => None,
        })
        .collect()
}

fn visit(
    table: &CoreTypeTable,
    id: CoreTypeId,
    seen: &mut [bool],
    next: &mut usize,
) -> Result<(), ExpressionError> {
    let index = id
        .index()
        .filter(|index| *index < seen.len())
        .ok_or_else(|| error("Core value references an unknown type", 0..0))?;
    if seen[index] {
        return Ok(());
    }
    for child in semantic::child_ids(table, index)? {
        visit(table, child, seen, next)?;
    }
    if index != *next {
        return Err(error("Core type table is not in first-use order", 0..0));
    }
    seen[index] = true;
    *next += 1;
    Ok(())
}
