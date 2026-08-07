use sha2::{Digest, Sha256};

use super::{bytes, length, tag, u16_value, u32_value};
use crate::program::expression::{CoreCallTarget, CoreInstructionKind, ValueId};

mod domain;
mod operators;
mod terminator;
pub(super) use terminator::encode as terminator;

pub(super) fn encode(digest: &mut Sha256, value: &CoreInstructionKind) {
    match value {
        CoreInstructionKind::Literal(value) => {
            tag(digest, 0x00);
            super::value::encode(digest, value);
        }
        CoreInstructionKind::Input(id) => tagged_u32(digest, 0x01, id.value()),
        CoreInstructionKind::Parameter(index) => tagged_usize(digest, 0x02, *index),
        CoreInstructionKind::Capture(index) => tagged_usize(digest, 0x03, *index),
        CoreInstructionKind::Closure {
            definition,
            captures,
        } => {
            tagged_u32(digest, 0x04, definition.value());
            ids(digest, captures);
        }
        CoreInstructionKind::Invoke { callee, arguments } => {
            tagged_u32(digest, 0x05, callee.value());
            ids(digest, arguments);
        }
        CoreInstructionKind::Unary { operator, operand } => {
            digest.update([0x06, operators::unary(*operator)]);
            u32_value(digest, operand.value());
        }
        CoreInstructionKind::Arithmetic {
            operator,
            left,
            right,
        } => binary(
            digest,
            0x07,
            operators::arithmetic(*operator),
            *left,
            *right,
        ),
        CoreInstructionKind::Compare {
            operator,
            left,
            right,
        } => binary(
            digest,
            0x08,
            operators::comparison(*operator),
            *left,
            *right,
        ),
        CoreInstructionKind::Equal {
            operator,
            left,
            right,
        } => binary(digest, 0x09, operators::equality(*operator), *left, *right),
        CoreInstructionKind::Call { target, arguments } => {
            tag(digest, 0x0a);
            match target {
                CoreCallTarget::Builtin(value) => bytes(digest, value.as_str().as_bytes()),
                CoreCallTarget::User(id) => digest.update(id.as_bytes()),
            }
            ids(digest, arguments);
        }
        CoreInstructionKind::List { elements } => tagged_ids(digest, 0x0b, elements),
        CoreInstructionKind::Tuple { elements } => tagged_ids(digest, 0x0c, elements),
        CoreInstructionKind::Range { start, end, step } => {
            tagged_u32(digest, 0x0d, start.value());
            u32_value(digest, end.value());
            option_id(digest, *step);
        }
        CoreInstructionKind::MapBegin { entries } => tagged_u32(digest, 0x0e, *entries),
        CoreInstructionKind::MapKey {
            builder,
            key,
            ordinal,
        } => {
            tagged_u32(digest, 0x0f, builder.value());
            u32_value(digest, key.value());
            u32_value(digest, *ordinal);
        }
        CoreInstructionKind::MapValue { pending, value } => {
            tagged_u32(digest, 0x10, pending.value());
            u32_value(digest, value.value());
        }
        CoreInstructionKind::MapFinish { builder } => tagged_u32(digest, 0x11, builder.value()),
        CoreInstructionKind::Collection {
            operation,
            iterable,
            initial,
            callable,
        } => {
            digest.update([0x12, operators::collection(*operation)]);
            u32_value(digest, iterable.value());
            option_id(digest, *initial);
            u32_value(digest, callable.value());
        }
        CoreInstructionKind::StructConstruct { type_id, fields } => {
            tag(digest, 0x13);
            digest.update(type_id.as_bytes());
            ids(digest, fields);
        }
        CoreInstructionKind::StructProject { structure, field } => {
            tagged_u32(digest, 0x14, structure.value());
            u16_value(digest, field.value());
        }
        CoreInstructionKind::EnumConstruct {
            type_id,
            variant,
            fields,
        } => {
            tag(digest, 0x15);
            digest.update(type_id.as_bytes());
            u16_value(digest, variant.value());
            ids(digest, fields);
        }
        CoreInstructionKind::DomainConstruct { opcode, operands } => {
            domain::encode(digest, 0x16, *opcode, operands)
        }
        CoreInstructionKind::GraphEmit { opcode, operands } => {
            domain::encode(digest, 0x17, *opcode, operands)
        }
        CoreInstructionKind::TemporalCompose {
            operation,
            operands,
        } => {
            digest.update([0x18, operators::compose(*operation)]);
            ids(digest, operands);
        }
        CoreInstructionKind::TemporalProject { operation, value } => {
            digest.update([0x19, operators::project(*operation)]);
            u32_value(digest, value.value());
        }
        CoreInstructionKind::LocalInit { slot, value } => {
            tagged_u32(digest, 0x1a, slot.value());
            u32_value(digest, value.value());
        }
        CoreInstructionKind::LocalSet { slot, value } => {
            tagged_u32(digest, 0x1b, slot.value());
            u32_value(digest, value.value());
        }
        CoreInstructionKind::LocalGet { slot } => tagged_u32(digest, 0x1c, slot.value()),
        CoreInstructionKind::TemporalAttach {
            kind,
            owner,
            selectors,
            animation,
        } => {
            digest.update([0x1d, *kind]);
            u32_value(digest, owner.value());
            ids(digest, selectors);
            u32_value(digest, animation.value());
        }
    }
}

fn binary(digest: &mut Sha256, kind: u8, operator: u8, left: ValueId, right: ValueId) {
    digest.update([kind, operator]);
    u32_value(digest, left.value());
    u32_value(digest, right.value());
}

fn tagged_ids(digest: &mut Sha256, kind: u8, values: &[ValueId]) {
    tag(digest, kind);
    ids(digest, values);
}

fn ids(digest: &mut Sha256, values: &[ValueId]) {
    length(digest, values.len());
    values
        .iter()
        .for_each(|value| u32_value(digest, value.value()));
}

fn option_id(digest: &mut Sha256, value: Option<ValueId>) {
    match value {
        Some(value) => tagged_u32(digest, 1, value.value()),
        None => tag(digest, 0),
    }
}

fn tagged_u32(digest: &mut Sha256, kind: u8, value: u32) {
    tag(digest, kind);
    u32_value(digest, value);
}

fn tagged_usize(digest: &mut Sha256, kind: u8, value: usize) {
    tag(digest, kind);
    length(digest, value);
}
