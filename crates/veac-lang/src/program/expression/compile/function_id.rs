use sha2::{Digest, Sha256};

use super::super::core::{CoreDigest, CoreProgram, FunctionId, FunctionRegistry, CORE_VERSION};
use super::super::{
    FunctionDefinition, FunctionEffect, MapKeyType, PrimitiveType, ValueType, ValueTypeKind,
};

const ID_DOMAIN: &[u8] = b"veac.function-identity.v5\0";
const CONTENT_DOMAIN: &[u8] = b"veac.function-content.v5\0";
const SYNTHETIC_NAMESPACE: &[u8] = b"veac:synthetic-api";

pub(super) fn identity(definition: &FunctionDefinition) -> FunctionId {
    let mut digest = Sha256::new();
    digest.update(ID_DOMAIN);
    digest.update(CORE_VERSION.to_be_bytes());
    match &definition.origin {
        Some(origin) => field(&mut digest, origin.source_id().as_bytes()),
        None => field(&mut digest, SYNTHETIC_NAMESPACE),
    }
    declaration(&mut digest, definition);
    FunctionId::from_digest(digest.finalize().into())
}

pub(super) fn content_body(
    source: &str,
    program: &CoreProgram,
    registry: &FunctionRegistry,
) -> CoreDigest {
    let mut digest = Sha256::new();
    digest.update(CONTENT_DOMAIN);
    digest.update(CORE_VERSION.to_be_bytes());
    field(&mut digest, source.as_bytes());
    for target in program.called_functions() {
        digest.update(target.as_bytes());
        let function = registry
            .get(target)
            .expect("typed call target is registered before content hashing");
        digest.update(function.content_digest().as_bytes());
    }
    CoreDigest::from_digest(digest.finalize().into())
}

fn declaration(digest: &mut Sha256, definition: &FunctionDefinition) {
    field(digest, definition.name.as_bytes());
    digest.update((definition.parameters.len() as u64).to_be_bytes());
    for parameter in &definition.parameters {
        field(digest, parameter.name.as_bytes());
        value_type(digest, &parameter.value_type);
    }
    value_type(digest, &definition.return_type);
}

fn field(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
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
        ValueTypeKind::List(element) => {
            digest.update([0x01]);
            value_type(digest, element);
        }
        ValueTypeKind::Range(element) => {
            digest.update([0x05]);
            value_type(digest, element);
        }
        ValueTypeKind::Map { key, value } => {
            digest.update([0x02, map_key_tag(key)]);
            value_type(digest, value);
        }
        ValueTypeKind::Tuple(elements) => {
            digest.update([0x03]);
            digest.update((elements.len() as u64).to_be_bytes());
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
            digest.update((parameters.len() as u64).to_be_bytes());
            for parameter in parameters {
                value_type(digest, parameter);
            }
            value_type(digest, result);
            digest.update([function_effect(effect)]);
        }
    }
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

#[cfg(test)]
#[path = "function_id/tests.rs"]
mod tests;
