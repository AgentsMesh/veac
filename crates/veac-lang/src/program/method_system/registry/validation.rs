use std::collections::BTreeSet;

use crate::program::expression::{ValueType, ValueTypeKind, MAX_FUNCTION_PARAMETERS};
use crate::program::{MethodDefinition, MethodRegistryError, TypeRegistry};

pub(super) fn definition(
    value: &MethodDefinition,
    types: &TypeRegistry,
) -> Result<(), MethodRegistryError> {
    let signature = value.signature();
    let receiver = signature.receiver().id();
    let nominal = types.definition(receiver).ok_or_else(|| {
        MethodRegistryError::new(
            "METHOD_UNKNOWN_TYPE",
            format!("method receiver refers to unknown TypeId {receiver}"),
        )
    })?;
    if nominal.canonical_source_id() != value.owner_source() {
        return Err(MethodRegistryError::new(
            "METHOD_FOREIGN_OWNER",
            format!("method owner does not define receiver TypeId {receiver}"),
        ));
    }
    if signature.function_id() != signature.expected_function_id() {
        return Err(MethodRegistryError::new(
            "METHOD_ID_MISMATCH",
            format!("method {} has a stale FunctionId", signature.name()),
        ));
    }
    signature_contract(value, types)?;
    if let Some(body) = value.body() {
        if body.origin().source_id() != value.owner_source() {
            return Err(MethodRegistryError::new(
                "METHOD_BODY_ORIGIN",
                "method body origin does not match its defining source",
            ));
        }
    }
    Ok(())
}

fn signature_contract(
    value: &MethodDefinition,
    types: &TypeRegistry,
) -> Result<(), MethodRegistryError> {
    let signature = value.signature();
    if !crate::name::is_name(signature.name()) || signature.name() == "self" {
        return Err(signature_error("method name is not a canonical identifier"));
    }
    let parameters = signature.parameters_with_receiver();
    let receiver_type = ValueType::nominal(signature.receiver().clone());
    if parameters.is_empty()
        || parameters[0].name != "self"
        || parameters[0].value_type != receiver_type
        || parameters.len() > MAX_FUNCTION_PARAMETERS
    {
        return Err(signature_error(
            "method signature must place its nominal self receiver at parameter zero",
        ));
    }
    let mut names = BTreeSet::from(["self"]);
    let mut found_default = false;
    for parameter in &parameters[1..] {
        if !crate::name::is_name(&parameter.name)
            || parameter.name == "self"
            || !names.insert(parameter.name.as_str())
        {
            return Err(signature_error(
                "method explicit parameter names must be unique canonical identifiers",
            ));
        }
        if parameter.has_default() {
            found_default = true;
        } else if found_default {
            return Err(signature_error(
                "required method parameters cannot follow a parameter with a default",
            ));
        }
        known_type(&parameter.value_type, types)?;
    }
    known_type(signature.return_type(), types)
}

fn known_type(value: &ValueType, types: &TypeRegistry) -> Result<(), MethodRegistryError> {
    let known = match value.kind() {
        ValueTypeKind::Primitive(_) | ValueTypeKind::Domain(_) => true,
        ValueTypeKind::Nominal(value) => types.definition(value.id()).is_some(),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => {
            known_type(value, types)?;
            true
        }
        ValueTypeKind::Tuple(values) => {
            values
                .iter()
                .try_for_each(|value| known_type(value, types))?;
            true
        }
        ValueTypeKind::Function {
            parameters, result, ..
        } => {
            parameters
                .iter()
                .chain(std::iter::once(result))
                .try_for_each(|value| known_type(value, types))?;
            true
        }
    };
    known.then_some(()).ok_or_else(|| {
        MethodRegistryError::new(
            "METHOD_UNKNOWN_TYPE",
            "method signature refers to an unknown nominal TypeId",
        )
    })
}

fn signature_error(message: &'static str) -> MethodRegistryError {
    MethodRegistryError::new("METHOD_SIGNATURE", message)
}
