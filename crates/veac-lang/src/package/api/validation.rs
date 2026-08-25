use std::collections::BTreeSet;

use super::{ApiEnumVariant, ApiExport, ApiField, ApiMetadataV1, ApiPrimitive, ApiType};
use crate::package::{PackageError, PackageErrorKind};

const MAX_EXPORTS: usize = 4096;
const MAX_NESTING: usize = 32;

mod callable;

pub(crate) fn metadata(value: &ApiMetadataV1) -> Result<(), PackageError> {
    if value.schema != super::PACKAGE_API_SCHEMA
        || value.schema_version != super::PACKAGE_API_SCHEMA_VERSION
    {
        return contract("unsupported package API schema identity");
    }
    if value.exports.len() > MAX_EXPORTS {
        return contract("package API exceeds the export limit");
    }
    if value.domain_capabilities.len() > crate::program::MAX_DOMAIN_OPERATIONS {
        return contract("package API exceeds the Domain capability limit");
    }
    let mut previous = None;
    for export in &value.exports {
        let key = export_key(export)?;
        if previous.as_ref().is_some_and(|item| item >= &key) {
            return contract("API exports must be unique and sorted by kind and name");
        }
        previous = Some(key);
        match export {
            ApiExport::Function {
                parameters,
                return_type,
                semantics,
                ..
            } => callable::validate(parameters, return_type, semantics, false)?,
            ApiExport::Method {
                receiver,
                parameters,
                return_type,
                semantics,
                ..
            } => {
                type_name(receiver)?;
                callable::validate(parameters, return_type, semantics, true)?;
            }
            ApiExport::Type { definition, .. } => match definition {
                super::ApiTypeDefinition::Struct { fields } => fields_valid(fields)?,
                super::ApiTypeDefinition::Enum { variants } => variants_valid(variants)?,
            },
            ApiExport::Constant {
                value_type: value, ..
            } => value_type(value, 0)?,
        }
    }
    let mut previous_capability = None;
    for capability in &value.domain_capabilities {
        if previous_capability.is_some_and(|opcode| opcode >= capability.opcode) {
            return contract("Domain capabilities must be unique and sorted by opcode");
        }
        let operation = crate::program::DomainOperationId::from_opcode(capability.opcode)
            .ok_or_else(|| error("Domain capability opcode is not in the current opset"))?;
        if capability.name != operation.name() {
            return contract("Domain capability name does not match its opcode");
        }
        previous_capability = Some(capability.opcode);
    }
    Ok(())
}

fn export_key(value: &ApiExport) -> Result<(u8, String), PackageError> {
    let (kind, name) = match value {
        ApiExport::Function { name, .. } => (0, name.clone()),
        ApiExport::Method { receiver, name, .. } => {
            type_name(receiver)?;
            name_valid(name)?;
            return Ok((1, format!("{}\0{}\0{name}", receiver.module, receiver.name)));
        }
        ApiExport::Type { name, .. } => (2, name.clone()),
        ApiExport::Constant { name, .. } => (3, name.clone()),
    };
    name_valid(&name)?;
    Ok((kind, name))
}

fn variants_valid(values: &[ApiEnumVariant]) -> Result<(), PackageError> {
    if values.is_empty() {
        return contract("exported enum must have at least one variant");
    }
    let mut names = BTreeSet::new();
    for value in values {
        name_valid(&value.name)?;
        if !names.insert(&value.name) {
            return contract("enum variant names must be unique");
        }
        fields_valid(&value.fields)?;
    }
    Ok(())
}

fn fields_valid(values: &[ApiField]) -> Result<(), PackageError> {
    named_types(values.iter().map(|field| (&field.name, &field.value_type)))
}

pub(super) fn named_types<'a>(
    values: impl Iterator<Item = (&'a String, &'a ApiType)>,
) -> Result<(), PackageError> {
    let mut names = BTreeSet::new();
    for (name, value) in values {
        name_valid(name)?;
        if !names.insert(name) {
            return contract("API field or parameter names must be unique");
        }
        value_type(value, 0)?;
    }
    Ok(())
}

pub(super) fn value_type(value: &ApiType, depth: usize) -> Result<(), PackageError> {
    if depth >= MAX_NESTING {
        return contract("API type exceeds the nesting limit");
    }
    match value {
        ApiType::Primitive { .. } => Ok(()),
        ApiType::Domain { name, opcode } => {
            let value = crate::program::DomainType::from_opcode(*opcode)
                .ok_or_else(|| error("Domain type opcode is not in the current opset"))?;
            (name == value.name())
                .then_some(())
                .ok_or_else(|| error("Domain type name does not match its opcode"))
        }
        ApiType::List { element } | ApiType::Map { value: element, .. } => {
            value_type(element, depth + 1)
        }
        ApiType::Range { element } if *element == ApiPrimitive::Int => Ok(()),
        ApiType::Range { .. } => contract("range API type only accepts int"),
        ApiType::Tuple { elements } if elements.len() >= 2 => elements
            .iter()
            .try_for_each(|value| value_type(value, depth + 1)),
        ApiType::Tuple { .. } => contract("tuple API type must have at least two elements"),
        ApiType::Function {
            parameters,
            return_type,
            ..
        } => {
            parameters
                .iter()
                .try_for_each(|value| value_type(value, depth + 1))?;
            value_type(return_type, depth + 1)
        }
        ApiType::Named { name } => type_name(name),
    }
}

fn type_name(value: &super::ApiTypeName) -> Result<(), PackageError> {
    crate::package::validation::source_path(&value.module)?;
    name_valid(&value.name)
}

fn name_valid(value: &str) -> Result<(), PackageError> {
    crate::name::is_name(value)
        .then_some(())
        .ok_or_else(|| error("API name is not a canonical VEAC name"))
}

pub(super) fn contract<T>(message: impl Into<String>) -> Result<T, PackageError> {
    Err(error(message))
}
fn error(message: impl Into<String>) -> PackageError {
    PackageError::new(PackageErrorKind::Contract, message)
}
