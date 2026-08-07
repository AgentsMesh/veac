use std::collections::BTreeMap;

use super::{SourceIndexBuildInput, SourceIndexBuildInputType};
use crate::program::expression::{PrimitiveType, ValueType, ValueTypeKind};
use crate::program::{BuildInputDeclaration, TypeDefinitionKind, TypeRegistry};

pub(crate) fn describe(
    declarations: &BTreeMap<String, BuildInputDeclaration>,
    registry: &TypeRegistry,
) -> Vec<SourceIndexBuildInput> {
    declarations
        .values()
        .map(|declaration| SourceIndexBuildInput {
            name: declaration.name().to_owned(),
            role: declaration.role(),
            value_type: describe_type(declaration.value_type(), registry),
        })
        .collect()
}

fn describe_type(value: &ValueType, registry: &TypeRegistry) -> SourceIndexBuildInputType {
    match value.kind() {
        ValueTypeKind::Primitive(primitive) => describe_primitive(primitive),
        ValueTypeKind::Nominal(reference) => {
            let definition = registry
                .definition(reference.id())
                .expect("verified Build input nominal type is registered");
            let TypeDefinitionKind::Enum(layout) = definition.kind() else {
                unreachable!("verified Build input nominal type is a payloadless enum")
            };
            SourceIndexBuildInputType::Enum {
                name: reference.to_string(),
                type_id: reference.id().to_string(),
                definition_sha256: definition.digest().to_string(),
                variants: layout
                    .variants()
                    .iter()
                    .map(|variant| variant.name().to_owned())
                    .collect(),
            }
        }
        _ => unreachable!("verified Build inputs have closed leaf types"),
    }
}

fn describe_primitive(value: PrimitiveType) -> SourceIndexBuildInputType {
    match value {
        PrimitiveType::Boolean => SourceIndexBuildInputType::Bool,
        PrimitiveType::Integer => SourceIndexBuildInputType::Integer,
        PrimitiveType::Scalar => SourceIndexBuildInputType::Scalar,
        PrimitiveType::Text => SourceIndexBuildInputType::Text,
        PrimitiveType::Time => SourceIndexBuildInputType::Time,
        PrimitiveType::Length => SourceIndexBuildInputType::Length,
        PrimitiveType::Angle => SourceIndexBuildInputType::Angle,
        PrimitiveType::Color => SourceIndexBuildInputType::Color,
        PrimitiveType::Identifier | PrimitiveType::Percent => {
            unreachable!("identifier and percent are not supported Build inputs")
        }
    }
}
