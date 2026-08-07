use sha2::{Digest, Sha256};

use crate::program::expression::{
    FunctionEffect, MapKeyType, PrimitiveType, ValueType, ValueTypeKind,
};

use super::{TypeDefinitionDigest, TypeDefinitionKind};

const DEFINITION_DOMAIN: &[u8] = b"veac.nominal-type-definition.v1\0";

pub(super) fn definition(kind: &TypeDefinitionKind) -> TypeDefinitionDigest {
    let mut digest = Sha256::new();
    digest.update(DEFINITION_DOMAIN);
    match kind {
        TypeDefinitionKind::Struct(value) => {
            digest.update([0x00]);
            fields(&mut digest, value.fields());
        }
        TypeDefinitionKind::Enum(value) => {
            digest.update([0x01]);
            count(&mut digest, value.variants().len());
            for variant in value.variants() {
                digest.update(variant.index().value().to_be_bytes());
                framed(&mut digest, variant.name().as_bytes());
                fields(&mut digest, variant.fields());
            }
        }
    }
    TypeDefinitionDigest::from_bytes(digest.finalize().into())
}

fn fields(digest: &mut Sha256, values: &[super::FieldDefinition]) {
    count(digest, values.len());
    for field in values {
        digest.update(field.index().value().to_be_bytes());
        framed(digest, field.name().as_bytes());
        value_type(digest, field.value_type());
    }
}

fn value_type(digest: &mut Sha256, value: &ValueType) {
    match value.kind() {
        ValueTypeKind::Primitive(kind) => digest.update([0x00, primitive_tag(kind)]),
        ValueTypeKind::Domain(value) => {
            digest.update([0x07]);
            digest.update(value.opcode().to_be_bytes());
        }
        ValueTypeKind::Nominal(value) => {
            digest.update([0x06]);
            digest.update(value.id().as_bytes());
        }
        ValueTypeKind::List(element) => nested(digest, 0x01, element),
        ValueTypeKind::Map { key, value } => {
            digest.update([0x02, map_key_tag(key)]);
            value_type(digest, value);
        }
        ValueTypeKind::Tuple(elements) => {
            digest.update([0x03]);
            count(digest, elements.len());
            for element in elements {
                value_type(digest, element);
            }
        }
        ValueTypeKind::Function {
            parameters,
            result,
            effect,
        } => {
            digest.update([0x04]);
            count(digest, parameters.len());
            for parameter in parameters {
                value_type(digest, parameter);
            }
            value_type(digest, result);
            digest.update([function_effect(effect)]);
        }
        ValueTypeKind::Range(element) => nested(digest, 0x05, element),
    }
}

fn nested(digest: &mut Sha256, tag: u8, value: &ValueType) {
    digest.update([tag]);
    value_type(digest, value);
}

fn primitive_tag(value: PrimitiveType) -> u8 {
    match value {
        PrimitiveType::Integer => 0x00,
        PrimitiveType::Scalar => 0x01,
        PrimitiveType::Time => 0x02,
        PrimitiveType::Length => 0x03,
        PrimitiveType::Percent => 0x04,
        PrimitiveType::Angle => 0x05,
        PrimitiveType::Text => 0x06,
        PrimitiveType::Color => 0x07,
        PrimitiveType::Boolean => 0x08,
        PrimitiveType::Identifier => 0x09,
    }
}

fn map_key_tag(value: MapKeyType) -> u8 {
    match value {
        MapKeyType::Text => 0x00,
        MapKeyType::Identifier => 0x01,
    }
}

fn function_effect(value: FunctionEffect) -> u8 {
    match value {
        FunctionEffect::Pure => 0,
        FunctionEffect::Local => 1,
        FunctionEffect::Emit => 2,
        FunctionEffect::Any => 3,
    }
}

fn count(digest: &mut Sha256, value: usize) {
    digest.update((value as u64).to_be_bytes());
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    count(digest, value.len());
    digest.update(value);
}
