use sha2::{Digest, Sha256};

use super::{length, tag};
use crate::program::expression::{
    FunctionEffect, MapKeyType, PrimitiveType, ValueType, ValueTypeKind,
};

pub(crate) fn encode(digest: &mut Sha256, value: &ValueType) {
    match value.kind() {
        ValueTypeKind::Primitive(value) => digest.update([0x00, primitive(value)]),
        ValueTypeKind::Domain(value) => {
            tag(digest, 0x07);
            digest.update(value.opcode().to_be_bytes());
        }
        ValueTypeKind::Nominal(value) => {
            tag(digest, 0x06);
            digest.update(value.id().as_bytes());
        }
        ValueTypeKind::List(element) => {
            tag(digest, 0x01);
            encode(digest, element);
        }
        ValueTypeKind::Map { key, value } => {
            digest.update([0x02, map_key(key)]);
            encode(digest, value);
        }
        ValueTypeKind::Tuple(elements) => {
            tag(digest, 0x03);
            length(digest, elements.len());
            elements.iter().for_each(|value| encode(digest, value));
        }
        ValueTypeKind::Function {
            parameters,
            result,
            effect,
        } => {
            tag(digest, 0x04);
            length(digest, parameters.len());
            parameters.iter().for_each(|value| encode(digest, value));
            encode(digest, result);
            digest.update([function_effect(effect)]);
        }
        ValueTypeKind::Range(element) => {
            tag(digest, 0x05);
            encode(digest, element);
        }
    }
}

fn primitive(value: PrimitiveType) -> u8 {
    match value {
        PrimitiveType::Integer => 0,
        PrimitiveType::Scalar => 1,
        PrimitiveType::Time => 2,
        PrimitiveType::Length => 3,
        PrimitiveType::Percent => 4,
        PrimitiveType::Angle => 5,
        PrimitiveType::Text => 6,
        PrimitiveType::Color => 7,
        PrimitiveType::Boolean => 8,
        PrimitiveType::Identifier => 9,
    }
}

fn map_key(value: MapKeyType) -> u8 {
    match value {
        MapKeyType::Text => 0,
        MapKeyType::Identifier => 1,
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
