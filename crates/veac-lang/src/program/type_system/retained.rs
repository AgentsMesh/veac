use std::mem::{size_of, size_of_val};

use crate::program::expression::{ValueType, ValueTypeKind};

use super::{TypeDefinition, TypeDefinitionKind, TypeRef};

const MAP_ENTRY_BYTES: usize = 64;

pub(super) fn definition(value: &TypeDefinition) -> Option<usize> {
    let mut bytes = size_of::<TypeDefinition>()
        .checked_add(value.canonical_source_id().len())?
        .checked_add(value.declared_name().len())?;
    bytes =
        bytes.checked_add(match value.kind() {
            TypeDefinitionKind::Struct(value) => fields(value.fields())?,
            TypeDefinitionKind::Enum(value) => value.variants().iter().try_fold(
                size_of_val(value.variants()),
                |bytes, variant| {
                    bytes
                        .checked_add(variant.name().len())?
                        .checked_add(fields(variant.fields())?)
                },
            )?,
        })?;
    bytes.checked_add(MAP_ENTRY_BYTES)
}

pub(super) fn binding(name: &str, value: &TypeRef) -> Option<usize> {
    MAP_ENTRY_BYTES
        .checked_add(name.len())?
        .checked_add(value.diagnostic_name().len())
}

fn fields(values: &[super::FieldDefinition]) -> Option<usize> {
    values.iter().try_fold(size_of_val(values), |bytes, field| {
        bytes
            .checked_add(field.name().len())?
            .checked_add(value_type(field.value_type())?)
    })
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
    values.iter().try_fold(size_of_val(values), |bytes, value| {
        bytes.checked_add(value_type(value)?)
    })
}
