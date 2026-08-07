use sha2::{Digest, Sha256};

use crate::program::expression::{
    FunctionEffect, FunctionId, FunctionParameter, MapKeyType, PrimitiveType, ValueType,
    ValueTypeKind, CORE_VERSION,
};
use crate::program::TypeId;

const ID_DOMAIN: &[u8] = b"veac.nominal-method-identity.v3\0";

pub(super) fn derive(
    receiver: TypeId,
    name: &str,
    parameters: &[FunctionParameter],
    result: &ValueType,
) -> FunctionId {
    let mut digest = Sha256::new();
    digest.update(ID_DOMAIN);
    digest.update(CORE_VERSION.to_be_bytes());
    digest.update(receiver.as_bytes());
    field(&mut digest, name.as_bytes());
    digest.update((parameters.len() as u64).to_be_bytes());
    for parameter in parameters {
        field(&mut digest, parameter.name.as_bytes());
        value_type(&mut digest, &parameter.value_type);
    }
    value_type(&mut digest, result);
    FunctionId::from_bytes(digest.finalize().into())
}

fn field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn value_type(digest: &mut Sha256, value: &ValueType) {
    match value.kind() {
        ValueTypeKind::Primitive(value) => digest.update([0x00, primitive(value)]),
        ValueTypeKind::Domain(value) => {
            digest.update([0x07]);
            digest.update(value.opcode().to_be_bytes());
        }
        ValueTypeKind::Nominal(value) => {
            digest.update([0x06]);
            digest.update(value.id().as_bytes());
        }
        ValueTypeKind::List(value) => nested(digest, 0x01, value),
        ValueTypeKind::Range(value) => nested(digest, 0x05, value),
        ValueTypeKind::Map { key, value } => {
            digest.update([0x02, map_key(key)]);
            value_type(digest, value);
        }
        ValueTypeKind::Tuple(values) => sequence(digest, 0x03, values),
        ValueTypeKind::Function {
            parameters,
            result,
            effect,
        } => {
            sequence(digest, 0x04, parameters);
            value_type(digest, result);
            digest.update([function_effect(effect)]);
        }
    }
}

fn nested(digest: &mut Sha256, tag: u8, value: &ValueType) {
    digest.update([tag]);
    value_type(digest, value);
}

fn sequence(digest: &mut Sha256, tag: u8, values: &[ValueType]) {
    digest.update([tag]);
    digest.update((values.len() as u64).to_be_bytes());
    values.iter().for_each(|value| value_type(digest, value));
}

fn function_effect(value: FunctionEffect) -> u8 {
    match value {
        FunctionEffect::Pure => 0,
        FunctionEffect::Local => 1,
        FunctionEffect::Emit => 2,
        FunctionEffect::Any => 3,
    }
}

fn primitive(value: PrimitiveType) -> u8 {
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

fn map_key(value: MapKeyType) -> u8 {
    match value {
        MapKeyType::Text => 0x00,
        MapKeyType::Identifier => 0x01,
    }
}
