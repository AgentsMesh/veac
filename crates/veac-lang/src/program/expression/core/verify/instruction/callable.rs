use std::ops::Range;

use super::super::definitions::Definitions;
use super::super::{error, type_table};
use crate::program::expression::core::{CoreProgram, ValueId};
use crate::program::expression::{ClosureDefinitionId, ExpressionError, ValueType, ValueTypeKind};

pub(super) fn closure(
    id: ClosureDefinitionId,
    captures: &[ValueId],
    program: &CoreProgram,
    definitions: &Definitions,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    let definition = id
        .index()
        .and_then(|index| program.closure_definitions.get(index))
        .filter(|definition| definition.id == id)
        .ok_or_else(|| {
            error(
                format!("unknown closure definition {}", id.value()),
                span.clone(),
            )
        })?;
    if captures.len() != definition.capture_types.len()
        || captures
            .iter()
            .zip(&definition.capture_types)
            .any(|(value, expected)| {
                value_type(program, definitions, *value, span.clone()) != Ok(expected)
            })
    {
        return Err(error("closure capture signature mismatch", span));
    }
    ValueType::function(
        definition.parameter_types.clone(),
        definition.body.result_type().clone(),
        definition.effect,
    )
    .map_err(|_| error("closure function type is invalid", 0..0))
}

pub(super) fn invoke(
    callee: ValueId,
    arguments: &[ValueId],
    program: &CoreProgram,
    definitions: &Definitions,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    let callable = value_type(program, definitions, callee, span.clone())?;
    let ValueTypeKind::Function {
        parameters, result, ..
    } = callable.kind()
    else {
        return Err(error("Invoke callee must have a function type", span));
    };
    if arguments.len() != parameters.len()
        || arguments.iter().zip(parameters).any(|(value, expected)| {
            value_type(program, definitions, *value, span.clone()) != Ok(expected)
        })
    {
        return Err(error("Invoke argument signature mismatch", span));
    }
    Ok(result.clone())
}

fn value_type<'a>(
    program: &'a CoreProgram,
    definitions: &Definitions,
    id: ValueId,
    span: Range<usize>,
) -> Result<&'a ValueType, ExpressionError> {
    type_table::value(program, definitions.type_id(id), span)
}
