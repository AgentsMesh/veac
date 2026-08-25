use std::mem::{size_of, size_of_val};

use super::{
    ModuleCallableSemantics, ModuleInterface, ModuleInterfaceType, ModuleParameterInterface,
    ModuleTypeDefinitionInterface,
};

pub(super) fn bytes(value: &ModuleInterface) -> Option<usize> {
    let mut bytes = size_of::<ModuleInterface>().checked_add(value.source_id.len())?;
    for function in &value.functions {
        bytes = bytes
            .checked_add(size_of_val(function))?
            .checked_add(function.name.len())?
            .checked_add(parameters(&function.parameters)?)?
            .checked_add(value_type(&function.return_type)?)?
            .checked_add(semantics(&function.semantics)?)?;
    }
    for method in &value.methods {
        bytes = bytes
            .checked_add(size_of_val(method))?
            .checked_add(method.receiver.source_id.len())?
            .checked_add(method.receiver.name.len())?
            .checked_add(method.name.len())?
            .checked_add(parameters(&method.parameters)?)?
            .checked_add(value_type(&method.return_type)?)?
            .checked_add(semantics(&method.semantics)?)?;
    }
    for nominal in &value.types {
        bytes = bytes
            .checked_add(size_of_val(nominal))?
            .checked_add(nominal.name.len())?
            .checked_add(definition(&nominal.definition)?)?;
    }
    for constant in &value.constants {
        bytes = bytes
            .checked_add(size_of_val(constant))?
            .checked_add(constant.name.len())?
            .checked_add(value_type(&constant.value_type)?)?;
    }
    value
        .domain_capabilities
        .iter()
        .try_fold(bytes, |bytes, value| {
            bytes
                .checked_add(size_of_val(value))?
                .checked_add(value.name.len())
        })
}

fn parameters(values: &[ModuleParameterInterface]) -> Option<usize> {
    values.iter().try_fold(size_of_val(values), |bytes, value| {
        bytes
            .checked_add(value.name.len())?
            .checked_add(value_type(&value.value_type)?)
    })
}

fn definition(value: &ModuleTypeDefinitionInterface) -> Option<usize> {
    match value {
        ModuleTypeDefinitionInterface::Struct { fields } => fields_bytes(fields),
        ModuleTypeDefinitionInterface::Enum { variants } => {
            variants
                .iter()
                .try_fold(size_of_val(variants.as_slice()), |bytes, variant| {
                    bytes
                        .checked_add(variant.name.len())?
                        .checked_add(fields_bytes(&variant.fields)?)
                })
        }
    }
}

fn fields_bytes(values: &[super::ModuleFieldInterface]) -> Option<usize> {
    values.iter().try_fold(size_of_val(values), |bytes, value| {
        bytes
            .checked_add(value.name.len())?
            .checked_add(value_type(&value.value_type)?)
    })
}

fn semantics(value: &ModuleCallableSemantics) -> Option<usize> {
    size_of::<ModuleCallableSemantics>().checked_add(
        value
            .result
            .parameters
            .len()
            .checked_mul(size_of::<super::ModuleParameterDependency>())?,
    )
}

fn value_type(value: &ModuleInterfaceType) -> Option<usize> {
    match value {
        ModuleInterfaceType::Primitive(_) | ModuleInterfaceType::Range(_) => Some(0),
        ModuleInterfaceType::Domain { name, .. } => Some(name.len()),
        ModuleInterfaceType::Named(value) => value.source_id.len().checked_add(value.name.len()),
        ModuleInterfaceType::List(value) => nested(value),
        ModuleInterfaceType::Map { value, .. } => nested(value),
        ModuleInterfaceType::Tuple(values) => sequence(values),
        ModuleInterfaceType::Function {
            parameters,
            return_type,
            ..
        } => sequence(parameters)?.checked_add(nested(return_type)?),
    }
}

fn nested(value: &ModuleInterfaceType) -> Option<usize> {
    size_of::<ModuleInterfaceType>().checked_add(value_type(value)?)
}

fn sequence(values: &[ModuleInterfaceType]) -> Option<usize> {
    values.iter().try_fold(size_of_val(values), |bytes, value| {
        bytes.checked_add(value_type(value)?)
    })
}
