use std::collections::BTreeSet;

use super::error;
use crate::program::expression::core::{CoreProgram, CoreTypeId, CoreTypeTable};
use crate::program::expression::{
    ExpressionError, ValueType, MAX_EXPRESSION_NODES, MAX_VALUE_TYPE_DEPTH,
};

mod order;
mod semantic;

const MAX_CORE_TYPES: usize = MAX_EXPRESSION_NODES * MAX_VALUE_TYPE_DEPTH;

pub(super) fn value(
    program: &CoreProgram,
    id: CoreTypeId,
    span: std::ops::Range<usize>,
) -> Result<&ValueType, ExpressionError> {
    program
        .types
        .value(id)
        .ok_or_else(|| error("Core position requires a known value type", span))
}

pub(super) fn core_type(
    program: &CoreProgram,
    id: CoreTypeId,
    span: std::ops::Range<usize>,
) -> Result<&crate::program::expression::CoreType, ExpressionError> {
    program
        .types
        .get(id)
        .ok_or_else(|| error("Core value references an unknown type", span))
}

pub(super) fn verify(program: &CoreProgram) -> Result<(), ExpressionError> {
    let entries = program.types.entries();
    if entries.is_empty() || entries.len() > MAX_CORE_TYPES {
        return Err(error("Core type table has an invalid size", 0..0));
    }
    let mut unique = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.id().index() != Some(index) {
            return Err(error("Core type IDs must be dense", 0..0));
        }
        if !unique.insert(entry.kind()) {
            return Err(error("Core type table contains a duplicate type", 0..0));
        }
        semantic::verify_entry(&program.types, index, entry.kind())?;
    }
    verify_positions(program)?;
    order::verify(program)
}

fn verify_positions(program: &CoreProgram) -> Result<(), ExpressionError> {
    require_value(&program.types, program.result_type, 0..0)?;
    for input in &program.inputs {
        require_value(&program.types, input.type_id, input.span.clone())?;
    }
    for block in &program.blocks {
        for parameter in &block.parameters {
            require_value(&program.types, parameter.type_id, parameter.span.clone())?;
        }
        for instruction in &block.instructions {
            core_type(program, instruction.type_id, instruction.span.clone())?;
        }
    }
    Ok(())
}

fn require_value(
    table: &CoreTypeTable,
    id: CoreTypeId,
    span: std::ops::Range<usize>,
) -> Result<(), ExpressionError> {
    table
        .value(id)
        .map(|_| ())
        .ok_or_else(|| error("Core position requires a known value type", span))
}
