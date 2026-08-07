use super::operand_type;
use crate::program::expression::core::{
    CoreInstruction, CoreProgram, CoreType, CoreTypeId, ValueId,
};
use crate::program::expression::{ExpressionError, MapKeyType, ValueType, ValueTypeKind};

use super::super::definitions::Definitions;
use super::super::{error, type_table};

pub(super) fn begin(
    instruction: &CoreInstruction,
    program: &CoreProgram,
) -> Result<(), ExpressionError> {
    match type_table::core_type(program, instruction.type_id, instruction.span.clone())? {
        CoreType::MapBuilder { .. } => Ok(()),
        _ => Err(error(
            "MapBegin must declare a map builder type",
            instruction.span.clone(),
        )),
    }
}

pub(super) fn key(
    instruction: &CoreInstruction,
    builder: ValueId,
    key: ValueId,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    let map_type = token_map(program, definitions, builder, true, instruction)?;
    require_declared(instruction, program, CoreType::MapPending { map_type })?;
    let (expected, _) = map_parts(program, map_type, instruction)?;
    let actual = operand_type(program, definitions, key, instruction)?;
    if actual.as_primitive() != Some(expected.primitive()) {
        return Err(error(
            "MapKey operand does not match the map key type",
            instruction.span.clone(),
        ));
    }
    Ok(())
}

pub(super) fn value(
    instruction: &CoreInstruction,
    pending: ValueId,
    value: ValueId,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    let map_type = token_map(program, definitions, pending, false, instruction)?;
    require_declared(instruction, program, CoreType::MapBuilder { map_type })?;
    let (_, expected) = map_parts(program, map_type, instruction)?;
    let actual = operand_type(program, definitions, value, instruction)?;
    if actual != expected {
        return Err(error(
            "MapValue operand does not match the map value type",
            instruction.span.clone(),
        ));
    }
    Ok(())
}

pub(super) fn finish(
    instruction: &CoreInstruction,
    builder: ValueId,
    program: &CoreProgram,
    definitions: &Definitions,
) -> Result<(), ExpressionError> {
    let map_type = token_map(program, definitions, builder, true, instruction)?;
    if instruction.type_id != map_type {
        return Err(error(
            "MapFinish must declare its builder's map type",
            instruction.span.clone(),
        ));
    }
    type_table::value(program, map_type, instruction.span.clone()).map(|_| ())
}

fn token_map(
    program: &CoreProgram,
    definitions: &Definitions,
    id: ValueId,
    builder: bool,
    instruction: &CoreInstruction,
) -> Result<CoreTypeId, ExpressionError> {
    let kind = type_table::core_type(program, definitions.type_id(id), instruction.span.clone())?;
    match (builder, kind) {
        (true, CoreType::MapBuilder { map_type }) | (false, CoreType::MapPending { map_type }) => {
            Ok(*map_type)
        }
        _ => Err(error(
            "map instruction consumes the wrong token type",
            instruction.span.clone(),
        )),
    }
}

fn require_declared(
    instruction: &CoreInstruction,
    program: &CoreProgram,
    expected: CoreType,
) -> Result<(), ExpressionError> {
    (type_table::core_type(program, instruction.type_id, instruction.span.clone())? == &expected)
        .then_some(())
        .ok_or_else(|| {
            error(
                "map instruction declares the wrong token type",
                instruction.span.clone(),
            )
        })
}

fn map_parts<'a>(
    program: &'a CoreProgram,
    id: CoreTypeId,
    instruction: &CoreInstruction,
) -> Result<(MapKeyType, &'a ValueType), ExpressionError> {
    match type_table::value(program, id, instruction.span.clone())?.kind() {
        ValueTypeKind::Map { key, value } => Ok((key, value)),
        _ => Err(error(
            "map token does not reference a map type",
            instruction.span.clone(),
        )),
    }
}
