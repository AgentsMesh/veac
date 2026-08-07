use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::super::digest;
use super::super::{
    TypeDefinition, TypeDefinitionKind, TypeId, TypeRegistryError, MAX_TYPE_MEMBERS,
};
use super::layout;

pub(super) fn definition(value: &TypeDefinition) -> Result<(), TypeRegistryError> {
    if !crate::name::is_name(value.declared_name()) {
        return Err(error(
            "TYPE_DECLARED_NAME",
            format!("invalid declared type name `{}`", value.declared_name()),
        ));
    }
    let expected = TypeId::derive(value.canonical_source_id(), value.declared_name());
    if value.type_ref().id() != expected {
        return Err(error(
            "TYPE_ID_MISMATCH",
            format!("type `{}` has a corrupted TypeId", value.declared_name()),
        ));
    }
    if value.digest() != digest::definition(value.kind()) {
        return Err(error(
            "TYPE_DEFINITION_DIGEST",
            format!(
                "type `{}` has a corrupted definition digest",
                value.declared_name()
            ),
        ));
    }
    match value.kind() {
        TypeDefinitionKind::Struct(value) => fields("struct", value.fields()),
        TypeDefinitionKind::Enum(value) => {
            if value.variants().is_empty() {
                return Err(error(
                    "TYPE_ENUM_EMPTY",
                    "enum must declare at least one variant",
                ));
            }
            members(
                "enum variant",
                value.variants().iter().map(|value| value.name()),
            )?;
            for (position, variant) in value.variants().iter().enumerate() {
                if variant.index().index() != position {
                    return Err(index("enum variant", variant.name()));
                }
                fields("enum payload", variant.fields())?;
            }
            Ok(())
        }
    }
}

pub(super) fn registry(
    values: &BTreeMap<TypeId, Arc<TypeDefinition>>,
) -> Result<(), TypeRegistryError> {
    for (id, definition) in values {
        if *id != definition.type_ref().id() {
            return Err(error(
                "TYPE_REGISTRY_KEY",
                format!("registry key {id} does not match its definition"),
            ));
        }
    }
    layout::verify(values)
}

fn fields(kind: &str, values: &[super::super::FieldDefinition]) -> Result<(), TypeRegistryError> {
    members(kind, values.iter().map(|value| value.name()))?;
    for (position, field) in values.iter().enumerate() {
        if field.index().index() != position {
            return Err(index(kind, field.name()));
        }
    }
    Ok(())
}

fn members<'a>(kind: &str, values: impl Iterator<Item = &'a str>) -> Result<(), TypeRegistryError> {
    let mut names = BTreeSet::new();
    for name in values {
        if names.len() >= MAX_TYPE_MEMBERS {
            return Err(error(
                "TYPE_MEMBER_LIMIT",
                format!("{kind} count exceeds {MAX_TYPE_MEMBERS}"),
            ));
        }
        if !crate::name::is_name(name) || !names.insert(name) {
            return Err(error(
                "TYPE_DUPLICATE_MEMBER",
                format!("invalid or duplicate {kind} `{name}`"),
            ));
        }
    }
    Ok(())
}

fn index(kind: &str, name: &str) -> TypeRegistryError {
    error(
        "TYPE_LAYOUT_INDEX",
        format!("{kind} `{name}` does not have its dense declaration index"),
    )
}

fn error(code: &'static str, message: impl Into<String>) -> TypeRegistryError {
    TypeRegistryError::new(code, message)
}
