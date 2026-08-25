use sha2::{Digest, Sha256};

use super::{CoreDigest, CoreProgram, CoreType};
use crate::program::expression::{FunctionEffect, Stage, ValueType};

mod input;
mod instruction;
#[cfg(test)]
mod tests;
mod value;
mod value_type;
pub(crate) use input::digest as input_declarations_digest;
pub(crate) use value_type::encode as encode_value_type;

const DOMAIN: &[u8] = b"veac.core-closure.v10-for-each\0";

pub(crate) fn closure_digest(
    parameters: &[ValueType],
    parameter_stages: &[Stage],
    captures: &[ValueType],
    effect: FunctionEffect,
    non_escaping: bool,
    body: &CoreProgram,
) -> CoreDigest {
    let mut digest = Sha256::new();
    digest.update(DOMAIN);
    types(&mut digest, parameters);
    stages(&mut digest, parameter_stages);
    types(&mut digest, captures);
    tag(&mut digest, function_effect_tag(effect));
    tag(&mut digest, u8::from(non_escaping));
    program(&mut digest, body);
    CoreDigest::from_digest(digest.finalize().into())
}

fn program(digest: &mut Sha256, value: &CoreProgram) {
    u16_value(digest, value.version);
    u16_value(digest, value.domain_opset.raw());
    digest.update(value.domain_registry_digest.as_bytes());
    u32_value(digest, value.entry.value());
    length(digest, value.types.entries().len());
    for entry in value.types.entries() {
        u32_value(digest, entry.id().value());
        match entry.kind() {
            CoreType::Value(value) => {
                tag(digest, 0x00);
                value_type::encode(digest, value);
            }
            CoreType::MapBuilder { map_type } => {
                tag(digest, 0x01);
                u32_value(digest, map_type.value());
            }
            CoreType::MapPending { map_type } => {
                tag(digest, 0x02);
                u32_value(digest, map_type.value());
            }
        }
    }
    length(digest, value.nominal_definitions.len());
    for entry in &value.nominal_definitions {
        digest.update(entry.definition().type_ref().id().as_bytes());
        digest.update(entry.definition().digest().as_bytes());
    }
    length(digest, value.inputs.len());
    for input in &value.inputs {
        u32_value(digest, input.id.value());
        bytes(digest, input.name.as_bytes());
        u32_value(digest, input.type_id.value());
        tag(digest, u8::from(input.trusted_function));
        match &input.callable {
            Some(value) => {
                tag(digest, 1);
                bytes(digest, value.definition_digest.as_bytes());
                types(digest, &value.capture_types);
            }
            None => tag(digest, 0),
        }
    }
    length(digest, value.local_slots.len());
    for slot in &value.local_slots {
        u32_value(digest, slot.id.value());
        u32_value(digest, slot.type_id.value());
    }
    length(digest, value.closure_definitions.len());
    for definition in &value.closure_definitions {
        u32_value(digest, definition.id.value());
        digest.update(
            closure_digest(
                &definition.parameter_types,
                &definition.parameter_stages,
                &definition.capture_types,
                definition.effect,
                definition.non_escaping,
                &definition.body,
            )
            .as_bytes(),
        );
    }
    length(digest, value.blocks.len());
    for block in &value.blocks {
        u32_value(digest, block.id.value());
        length(digest, block.parameters.len());
        for parameter in &block.parameters {
            u32_value(digest, parameter.id.value());
            u32_value(digest, parameter.type_id.value());
        }
        length(digest, block.instructions.len());
        for instruction in &block.instructions {
            u32_value(digest, instruction.id.value());
            u32_value(digest, instruction.type_id.value());
            instruction::encode(digest, &instruction.kind);
        }
        instruction::terminator(digest, &block.terminator);
    }
    u32_value(digest, value.result_type.value());
}

fn function_effect_tag(effect: FunctionEffect) -> u8 {
    match effect {
        FunctionEffect::Pure => 0,
        FunctionEffect::Local => 1,
        FunctionEffect::Emit => 2,
        FunctionEffect::Any => 3,
    }
}

fn types(digest: &mut Sha256, values: &[ValueType]) {
    length(digest, values.len());
    values
        .iter()
        .for_each(|value| value_type::encode(digest, value));
}

fn stages(digest: &mut Sha256, values: &[Stage]) {
    length(digest, values.len());
    values.iter().for_each(|value| {
        tag(
            digest,
            match value {
                Stage::Const => 0,
                Stage::Build => 1,
                Stage::Temporal => 2,
            },
        );
    });
}

pub(super) fn tag(digest: &mut Sha256, value: u8) {
    digest.update([value]);
}

pub(super) fn length(digest: &mut Sha256, value: usize) {
    digest.update((value as u64).to_be_bytes());
}

pub(super) fn u16_value(digest: &mut Sha256, value: u16) {
    digest.update(value.to_be_bytes());
}

pub(super) fn u32_value(digest: &mut Sha256, value: u32) {
    digest.update(value.to_be_bytes());
}

pub(super) fn bytes(digest: &mut Sha256, value: &[u8]) {
    length(digest, value.len());
    digest.update(value);
}
