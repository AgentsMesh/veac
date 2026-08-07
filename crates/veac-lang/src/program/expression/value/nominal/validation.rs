use super::{EnumValue, StructValue};
use crate::program::expression::value::{Value, ValueConstructionError};
use crate::program::{FieldDefinition, TypeDefinition, TypeDefinitionKind, TypeRegistry};

pub(super) fn validate_fields(
    registry: &TypeRegistry,
    layout: &[FieldDefinition],
    fields: &[Value],
) -> Result<(), ValueConstructionError> {
    if fields.len() != layout.len() {
        return Err(error(
            "VALUE_NOMINAL_FIELD_COUNT",
            "nominal field count does not match its verified layout",
        ));
    }
    for (field, value) in layout.iter().zip(fields) {
        if &value.value_type() != field.value_type() {
            return Err(error(
                "VALUE_NOMINAL_FIELD_TYPE",
                format!("nominal field `{}` has the wrong value type", field.name()),
            ));
        }
        validate_registry(value, registry)?;
    }
    super::super::domain_affinity::validate(fields)?;
    Ok(())
}

pub(crate) fn validate_registry(
    value: &Value,
    registry: &TypeRegistry,
) -> Result<(), ValueConstructionError> {
    match value {
        Value::Struct(value) => structure(value, registry),
        Value::Enum(value) => enumeration(value, registry),
        Value::List(value) => values(value.values(), registry),
        Value::Tuple(value) => values(value.values(), registry),
        Value::Map(value) => {
            for entry in value.entries() {
                validate_registry(entry.key(), registry)?;
                validate_registry(entry.value(), registry)?;
            }
            Ok(())
        }
        // Hidden captures are governed by the closure's verified lexical program.
        Value::Closure(value) => {
            values(value.captures(), value.definition().body().nominal_types())
        }
        Value::Domain(_) => Ok(()),
        _ => Ok(()),
    }
}

fn structure(value: &StructValue, registry: &TypeRegistry) -> Result<(), ValueConstructionError> {
    let definition = current(value.definition(), registry)?;
    let TypeDefinitionKind::Struct(layout) = definition.kind() else {
        return Err(error(
            "VALUE_NOMINAL_KIND",
            "nominal struct value has a non-struct definition",
        ));
    };
    validate_fields(registry, layout.fields(), value.fields())
}

fn enumeration(value: &EnumValue, registry: &TypeRegistry) -> Result<(), ValueConstructionError> {
    let definition = current(value.definition(), registry)?;
    let TypeDefinitionKind::Enum(layout) = definition.kind() else {
        return Err(error(
            "VALUE_NOMINAL_KIND",
            "nominal enum value has a non-enum definition",
        ));
    };
    let variant = layout
        .variants()
        .get(value.variant().index())
        .ok_or_else(|| {
            error(
                "VALUE_NOMINAL_VARIANT",
                "enum variant index is outside its verified layout",
            )
        })?;
    validate_fields(registry, variant.fields(), value.fields())
}

fn current<'a>(
    value: &TypeDefinition,
    registry: &'a TypeRegistry,
) -> Result<&'a TypeDefinition, ValueConstructionError> {
    registry
        .definition(value.type_ref().id())
        .filter(|current| current.digest() == value.digest())
        .ok_or_else(|| {
            error(
                "VALUE_NOMINAL_DEFINITION",
                "nominal value layout does not match the verified type registry",
            )
        })
}

fn values(values: &[Value], registry: &TypeRegistry) -> Result<(), ValueConstructionError> {
    values
        .iter()
        .try_for_each(|value| validate_registry(value, registry))
}

fn error(code: &'static str, message: impl Into<String>) -> ValueConstructionError {
    ValueConstructionError::new(code, message)
}

#[cfg(test)]
#[path = "validation/tests.rs"]
mod tests;
