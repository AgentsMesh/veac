use sha2::{Digest, Sha256};

use veac_lang_model::{Effect, PrimitiveType, Stage};

use crate::{
    DomainInstructionKind, DomainOperationExposure, DomainPluginIdentity, DomainRuntimeAction,
    DomainValueShape, OperandAxis, TemporalLoweringOpcode,
};

pub(super) fn builtin_plugins() -> [DomainPluginIdentity<'static>; 1] {
    [DomainPluginIdentity::new(
        "video.plugin.veac.reference_monochrome.v1.e208978cbc18912f43b3501f09ea1236c083f6c871567b824655000e7a02e4d9",
        "e208978cbc18912f43b3501f09ea1236c083f6c871567b824655000e7a02e4d9",
    )]
}

pub(super) fn plugin_identities<'a>(
    digest: &mut Sha256,
    values: impl IntoIterator<Item = DomainPluginIdentity<'a>>,
) {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort_by_key(|value| value.effect_type());
    digest.update((values.len() as u64).to_be_bytes());
    for value in values {
        framed(digest, value.effect_type().as_bytes());
        framed(digest, value.digest().as_bytes());
    }
}

pub(super) fn exposure(digest: &mut Sha256, value: &DomainOperationExposure) {
    match value {
        DomainOperationExposure::FreeFunction { name } => {
            digest.update([0x00]);
            framed(digest, name.as_bytes());
        }
        DomainOperationExposure::Method { receiver, name } => {
            digest.update([0x01]);
            digest.update(receiver.opcode().to_be_bytes());
            framed(digest, name.as_bytes());
        }
    }
}

pub(super) fn shape(digest: &mut Sha256, value: DomainValueShape) {
    match value {
        DomainValueShape::Primitive(value) => digest.update([0x00, primitive(value)]),
        DomainValueShape::PrimitiveList(value) => digest.update([0x03, primitive(value)]),
        DomainValueShape::Domain(value) => {
            digest.update([0x01]);
            digest.update(value.opcode().to_be_bytes());
        }
        DomainValueShape::DomainList(value) => {
            digest.update([0x02]);
            digest.update(value.opcode().to_be_bytes());
        }
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

pub(super) const fn instruction(value: DomainInstructionKind) -> u8 {
    match value {
        DomainInstructionKind::DomainConstruct => 0x00,
        DomainInstructionKind::GraphEmit => 0x01,
    }
}

pub(super) const fn runtime_action(value: DomainRuntimeAction) -> u8 {
    match value {
        DomainRuntimeAction::Description => 0x00,
        DomainRuntimeAction::EntityConstructor => 0x01,
        DomainRuntimeAction::OwnedAttachment => 0x02,
        DomainRuntimeAction::NonOwningUpdate => 0x03,
        DomainRuntimeAction::ProjectEntry => 0x04,
        DomainRuntimeAction::RelationConstructor => 0x05,
    }
}

pub(super) const fn effect(value: Effect) -> u8 {
    match value {
        Effect::Pure => 0x00,
        Effect::LocalMutation => 0x01,
        Effect::GraphEmit => 0x02,
    }
}

pub(super) const fn axis(value: OperandAxis) -> u8 {
    match value {
        OperandAxis::Topology => 0x00,
        OperandAxis::Leaf => 0x01,
    }
}

pub(super) const fn max_stage(value: Stage) -> u8 {
    match value {
        Stage::Build => 0x00,
        Stage::Temporal => 0x01,
        Stage::Const => unreachable!(),
    }
}

pub(super) fn temporal_lowering(digest: &mut Sha256, value: Option<TemporalLoweringOpcode>) {
    let bytes = match value {
        None => &[0x00][..],
        Some(TemporalLoweringOpcode::ComposeVector) => &[0x01, 0x00],
        Some(TemporalLoweringOpcode::ComposePoint) => &[0x01, 0x01],
        Some(TemporalLoweringOpcode::ComposeRect) => &[0x01, 0x02],
    };
    digest.update(bytes);
}

pub(super) fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
