use std::mem::size_of;

use crate::program::expression::{ValueType, ValueTypeKind};

use super::MethodDefinition;

const MAP_ENTRY_BYTES: usize = 64;

pub(super) fn definition(value: &MethodDefinition) -> Option<usize> {
    let signature = value.signature();
    let mut bytes = size_of::<MethodDefinition>()
        .checked_add(MAP_ENTRY_BYTES)?
        .checked_add(value.owner_source().len())?
        .checked_add(signature.name().len())?;
    for parameter in signature.parameters_with_receiver() {
        bytes = bytes
            .checked_add(size_of_val(parameter))?
            .checked_add(parameter.name.len())?
            .checked_add(value_type(&parameter.value_type)?)?;
        if let Some(default) = parameter.default() {
            bytes = bytes.checked_add(default.source().len())?;
            if let Some(origin) = default.origin() {
                bytes = bytes.checked_add(origin.source_id().len())?;
            }
        }
    }
    bytes = bytes.checked_add(value_type(signature.return_type())?)?;
    if let Some(body) = value.body() {
        bytes = bytes
            .checked_add(body.source().len())?
            .checked_add(body.origin().source_id().len())?;
    }
    Some(bytes)
}

fn value_type(value: &ValueType) -> Option<usize> {
    match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => Some(0),
        ValueTypeKind::Nominal(value) => Some(value.diagnostic_name().len()),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => {
            size_of::<ValueType>().checked_add(value_type(value)?)
        }
        ValueTypeKind::Tuple(values) => sequence(values),
        ValueTypeKind::Function {
            parameters, result, ..
        } => sequence(parameters)?.checked_add(size_of::<ValueType>() + value_type(result)?),
    }
}

fn sequence(values: &[ValueType]) -> Option<usize> {
    values.iter().try_fold(
        size_of::<ValueType>().checked_mul(values.len())?,
        |bytes, value| bytes.checked_add(value_type(value)?),
    )
}
