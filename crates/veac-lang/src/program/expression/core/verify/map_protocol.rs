use super::error;
use crate::program::expression::core::{CoreInstruction, CoreInstructionKind, CoreProgram};
use crate::program::expression::ExpressionError;

mod graph;
#[cfg(test)]
mod tests;
mod uses;

use uses::{Consumers, InstructionUse};

pub(super) fn verify(program: &CoreProgram) -> Result<(), ExpressionError> {
    let consumers = Consumers::collect(program);
    let mut visited = vec![false; program.value_count()];
    for instruction in program
        .blocks
        .iter()
        .flat_map(|block| block.instructions.iter())
    {
        if matches!(instruction.kind, CoreInstructionKind::MapBegin { .. }) {
            chain(program, instruction, &consumers, &mut visited)?;
        }
    }
    for block in &program.blocks {
        for instruction in &block.instructions {
            let internal = program
                .types
                .get(instruction.type_id)
                .is_some_and(crate::program::expression::CoreType::is_internal);
            let index = instruction.id.index().expect("verified value ID");
            if internal && !visited[index] {
                return Err(error(
                    "map token is not rooted in a MapBegin chain",
                    instruction.span.clone(),
                ));
            }
        }
    }
    Ok(())
}

fn chain(
    program: &CoreProgram,
    begin: &CoreInstruction,
    consumers: &Consumers<'_>,
    visited: &mut [bool],
) -> Result<(), ExpressionError> {
    let CoreInstructionKind::MapBegin { entries } = begin.kind else {
        unreachable!("map protocol roots are MapBegin instructions")
    };
    let entries = usize::try_from(entries).map_err(|_| {
        error(
            "map entry count does not fit this target",
            begin.span.clone(),
        )
    })?;
    let required = entries
        .checked_mul(2)
        .and_then(|value| value.checked_add(1));
    if required.is_none_or(|value| value > program.value_count()) {
        return Err(error(
            "MapBegin entry count exceeds available Core instructions",
            begin.span.clone(),
        ));
    }
    let mut builder = begin.id;
    mark(builder, visited, begin)?;
    for expected in 0..entries {
        let key = consume(program, consumers, builder)?;
        let CoreInstructionKind::MapKey {
            builder: operand,
            ordinal,
            ..
        } = key.instruction.kind
        else {
            return Err(error(
                "map builder must be consumed by MapKey",
                key.instruction.span.clone(),
            ));
        };
        if operand != builder || usize::try_from(ordinal).ok() != Some(expected) {
            return Err(error(
                "MapKey ordinals must be dense and ordered",
                key.instruction.span.clone(),
            ));
        }
        let pending = key.instruction.id;
        mark(pending, visited, key.instruction)?;
        let value = consume(program, consumers, pending)?;
        let CoreInstructionKind::MapValue {
            pending: operand, ..
        } = value.instruction.kind
        else {
            return Err(error(
                "pending map key must be consumed by MapValue",
                value.instruction.span.clone(),
            ));
        };
        if operand != pending {
            return Err(error(
                "MapValue must consume the preceding pending token",
                value.instruction.span.clone(),
            ));
        }
        builder = value.instruction.id;
        mark(builder, visited, value.instruction)?;
    }
    let finish = consume(program, consumers, builder)?;
    match finish.instruction.kind {
        CoreInstructionKind::MapFinish { builder: operand } if operand == builder => Ok(()),
        _ => Err(error(
            "map builder entry count must end exactly at MapFinish",
            finish.instruction.span.clone(),
        )),
    }
}

fn consume<'a>(
    program: &CoreProgram,
    consumers: &'a Consumers<'a>,
    token: crate::program::expression::ValueId,
) -> Result<InstructionUse<'a>, ExpressionError> {
    let usage = consumers.single(token)?;
    let definition = consumers.definition_block(token);
    if !graph::postdominates(program, definition, usage.block) {
        return Err(error(
            "map token consumption must postdominate its definition",
            usage.instruction.span.clone(),
        ));
    }
    Ok(usage)
}

fn mark(
    id: crate::program::expression::ValueId,
    visited: &mut [bool],
    instruction: &CoreInstruction,
) -> Result<(), ExpressionError> {
    let index = id.index().expect("verified map token ID");
    if std::mem::replace(&mut visited[index], true) {
        return Err(error(
            "map token belongs to more than one construction chain",
            instruction.span.clone(),
        ));
    }
    Ok(())
}
