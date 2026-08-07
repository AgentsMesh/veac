use sha2::Sha256;

use super::{ids, tagged_u32};
use crate::program::expression::core::digest::{length, tag, u16_value, u32_value};
use crate::program::expression::{
    CoreForEachOrder, CoreForEachSlot, CoreForEachSlotId, CoreTerminator, Effect,
};

pub(crate) fn encode(digest: &mut Sha256, value: &CoreTerminator) {
    match value {
        CoreTerminator::Return { value, .. } => tagged_u32(digest, 0x00, value.value()),
        CoreTerminator::Jump {
            target, arguments, ..
        } => {
            tagged_u32(digest, 0x01, target.value());
            ids(digest, arguments);
        }
        CoreTerminator::Branch {
            condition,
            then_target,
            else_target,
            ..
        } => {
            tagged_u32(digest, 0x02, condition.value());
            u32_value(digest, then_target.value());
            u32_value(digest, else_target.value());
        }
        CoreTerminator::Match {
            scrutinee, arms, ..
        } => {
            tagged_u32(digest, 0x03, scrutinee.value());
            length(digest, arms.len());
            for arm in arms {
                u16_value(digest, arm.variant().value());
                u32_value(digest, arm.target().value());
            }
        }
        CoreTerminator::ForEach(value) => {
            tagged_u32(digest, 0x04, value.iterable().value());
            u32_value(digest, value.maximum_count());
            tag(
                digest,
                match value.order() {
                    CoreForEachOrder::Source => 0,
                },
            );
            slot(digest, value.element());
            slot(digest, value.index());
            u32_value(digest, value.body().value());
            ids(digest, value.captures());
            u32_value(digest, value.continuation().value());
            let provenance = value.provenance();
            u32_value(digest, provenance.definition().value());
            span(digest, provenance.loop_span());
            span(digest, provenance.binding_span());
            tag(
                digest,
                match value.effect().summary() {
                    Effect::Pure => 0,
                    Effect::LocalMutation => 1,
                    Effect::GraphEmit => 2,
                },
            );
            tag(digest, u8::from(value.effect().contains_local_mutation()));
        }
    }
}

fn slot(digest: &mut Sha256, value: CoreForEachSlot) {
    tag(
        digest,
        match value.id() {
            CoreForEachSlotId::Element => 0,
            CoreForEachSlotId::Index => 1,
        },
    );
    u32_value(digest, value.type_id().value());
}

fn span(digest: &mut Sha256, value: &std::ops::Range<usize>) {
    length(digest, value.start);
    length(digest, value.end);
}
