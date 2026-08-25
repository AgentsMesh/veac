use super::super::super::expression::{ValueType, ValueTypeKind};
use super::super::super::{Diagnostics, TypeDefinition, TypeDefinitionKind, TypeId, TypeRegistry};
use super::super::model::*;

pub(super) fn type_interface(
    name: &str,
    definition: &TypeDefinition,
    types: &TypeRegistry,
) -> Result<ModuleTypeInterface, Diagnostics> {
    let definition = match definition.kind() {
        TypeDefinitionKind::Struct(value) => ModuleTypeDefinitionInterface::Struct {
            fields: fields(value.fields(), types)?,
        },
        TypeDefinitionKind::Enum(value) => ModuleTypeDefinitionInterface::Enum {
            variants: value
                .variants()
                .iter()
                .map(|variant| {
                    Ok(ModuleEnumVariantInterface {
                        name: variant.name().to_owned(),
                        fields: fields(variant.fields(), types)?,
                    })
                })
                .collect::<Result<Vec<_>, Diagnostics>>()?,
        },
    };
    Ok(ModuleTypeInterface {
        name: name.to_owned(),
        definition,
    })
}

fn fields(
    values: &[super::super::super::FieldDefinition],
    types: &TypeRegistry,
) -> Result<Vec<ModuleFieldInterface>, Diagnostics> {
    values
        .iter()
        .map(|field| {
            Ok(ModuleFieldInterface {
                name: field.name().to_owned(),
                value_type: value_type(field.value_type(), types)?,
            })
        })
        .collect()
}

pub(super) fn value_type(
    value: &ValueType,
    types: &TypeRegistry,
) -> Result<ModuleInterfaceType, Diagnostics> {
    Ok(match value.kind() {
        ValueTypeKind::Primitive(value) => ModuleInterfaceType::Primitive(value),
        ValueTypeKind::Domain(value) => ModuleInterfaceType::Domain {
            name: value.name().to_owned(),
            opcode: value.opcode(),
        },
        ValueTypeKind::Nominal(value) => {
            ModuleInterfaceType::Named(nominal_name(value.id(), types)?)
        }
        ValueTypeKind::List(value) => {
            ModuleInterfaceType::List(Box::new(value_type(value, types)?))
        }
        ValueTypeKind::Range(value) => ModuleInterfaceType::Range(
            value
                .as_primitive()
                .expect("verified range interface has a primitive element"),
        ),
        ValueTypeKind::Map { key, value } => ModuleInterfaceType::Map {
            key,
            value: Box::new(value_type(value, types)?),
        },
        ValueTypeKind::Tuple(values) => ModuleInterfaceType::Tuple(
            values
                .iter()
                .map(|value| value_type(value, types))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        ValueTypeKind::Function {
            parameters,
            result,
            effect,
        } => ModuleInterfaceType::function(
            parameters
                .iter()
                .map(|value| value_type(value, types))
                .collect::<Result<Vec<_>, _>>()?,
            value_type(result, types)?,
            effect,
        ),
    })
}

pub(super) fn nominal_name(
    id: TypeId,
    types: &TypeRegistry,
) -> Result<ModuleTypeName, Diagnostics> {
    let definition = types
        .definition(id)
        .expect("verified interface nominal type has a definition");
    Ok(ModuleTypeName {
        source_id: definition.canonical_source_id().to_owned(),
        name: definition.declared_name().to_owned(),
    })
}
