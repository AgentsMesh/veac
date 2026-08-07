use super::{ValueType, ValueTypeKind};
use crate::program::{TypeDefinition, TypeDefinitionKind, TypeRegistry};

impl ValueType {
    pub fn contains_domain_in(&self, registry: &TypeRegistry) -> Option<bool> {
        contains_type(self, registry)
    }

    pub fn supports_equality_in(&self, registry: &TypeRegistry) -> Option<bool> {
        let functions = self.contains_function_in(registry)?;
        let domains = self.contains_domain_in(registry)?;
        Some(!functions && !domains)
    }

    pub fn is_public_input_in(&self, registry: &TypeRegistry) -> Option<bool> {
        self.contains_domain_in(registry).map(|value| !value)
    }
}

fn contains_type(value: &ValueType, registry: &TypeRegistry) -> Option<bool> {
    match value.kind() {
        ValueTypeKind::Domain(_) => Some(true),
        ValueTypeKind::Primitive(_) => Some(false),
        ValueTypeKind::Nominal(value) => contains_nominal(value.id(), registry),
        ValueTypeKind::List(value)
        | ValueTypeKind::Range(value)
        | ValueTypeKind::Map { value, .. } => contains_type(value, registry),
        ValueTypeKind::Tuple(values) => values
            .iter()
            .map(|value| contains_type(value, registry))
            .try_fold(false, merge),
        ValueTypeKind::Function {
            parameters, result, ..
        } => parameters
            .iter()
            .chain(std::iter::once(result))
            .map(|value| contains_type(value, registry))
            .try_fold(false, merge),
    }
}

fn contains_nominal(id: crate::program::TypeId, registry: &TypeRegistry) -> Option<bool> {
    let definition = registry.definition(id)?;
    let result = fields(definition)
        .map(|value| contains_type(value, registry))
        .try_fold(false, merge);
    result
}

fn fields(value: &TypeDefinition) -> Box<dyn Iterator<Item = &ValueType> + '_> {
    match value.kind() {
        TypeDefinitionKind::Struct(value) => {
            Box::new(value.fields().iter().map(|field| field.value_type()))
        }
        TypeDefinitionKind::Enum(value) => Box::new(
            value
                .variants()
                .iter()
                .flat_map(|variant| variant.fields())
                .map(|field| field.value_type()),
        ),
    }
}

fn merge(found: bool, value: Option<bool>) -> Option<bool> {
    value.map(|value| found || value)
}
