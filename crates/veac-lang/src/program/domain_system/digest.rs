use sha2::{Digest, Sha256};

use crate::program::expression::{Effect, PrimitiveType};

use super::{
    DomainInstructionKind, DomainOperationContract, DomainOperationExposure, DomainOpsetVersion,
    DomainRuntimeAction, DomainValueShape, OperandAxis, TemporalLoweringOpcode,
};

const DIGEST_DOMAIN: &[u8] = b"veac.domain-operation-registry.v5\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DomainRegistryDigest([u8; 32]);

impl DomainRegistryDigest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl std::fmt::Display for DomainRegistryDigest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

pub(super) fn registry<'a>(
    version: DomainOpsetVersion,
    values: impl ExactSizeIterator<Item = &'a DomainOperationContract>,
) -> DomainRegistryDigest {
    let mut digest = Sha256::new();
    digest.update(DIGEST_DOMAIN);
    digest.update(version.raw().to_be_bytes());
    digest.update((values.len() as u64).to_be_bytes());
    for value in values {
        digest.update(value.id().opcode().to_be_bytes());
        framed(&mut digest, value.name().as_bytes());
        exposure(&mut digest, value.exposure());
        digest.update([
            instruction(value.instruction()),
            runtime_action(value.runtime_action()),
            effect(value.effect()),
            max_stage(value.max_stage()),
        ]);
        temporal_lowering(&mut digest, value.temporal_lowering());
        digest.update((value.operands().len() as u64).to_be_bytes());
        for operand in value.operands() {
            framed(&mut digest, operand.name().as_bytes());
            shape(&mut digest, operand.shape());
            digest.update([axis(operand.axis())]);
        }
        shape(&mut digest, value.result());
    }
    plugins(&mut digest);
    DomainRegistryDigest::from_bytes(digest.finalize().into())
}

fn plugins(digest: &mut Sha256) {
    plugin_identities(digest, veac_ir::plugin_effects().to_vec());
}

fn plugin_identities(digest: &mut Sha256, mut values: Vec<veac_ir::PluginEffectDescriptor>) {
    values.sort_by_key(|value| value.effect_type);
    digest.update((values.len() as u64).to_be_bytes());
    for value in values {
        framed(digest, value.effect_type.as_bytes());
        framed(digest, value.digest.as_bytes());
    }
}

fn exposure(digest: &mut Sha256, value: &DomainOperationExposure) {
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

fn shape(digest: &mut Sha256, value: DomainValueShape) {
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

const fn instruction(value: DomainInstructionKind) -> u8 {
    match value {
        DomainInstructionKind::DomainConstruct => 0x00,
        DomainInstructionKind::GraphEmit => 0x01,
    }
}

const fn runtime_action(value: DomainRuntimeAction) -> u8 {
    match value {
        DomainRuntimeAction::Description => 0x00,
        DomainRuntimeAction::EntityConstructor => 0x01,
        DomainRuntimeAction::OwnedAttachment => 0x02,
        DomainRuntimeAction::NonOwningUpdate => 0x03,
        DomainRuntimeAction::ProjectEntry => 0x04,
        DomainRuntimeAction::RelationConstructor => 0x05,
    }
}

const fn effect(value: Effect) -> u8 {
    match value {
        Effect::Pure => 0x00,
        Effect::LocalMutation => 0x01,
        Effect::GraphEmit => 0x02,
    }
}

const fn axis(value: OperandAxis) -> u8 {
    match value {
        OperandAxis::Topology => 0x00,
        OperandAxis::Leaf => 0x01,
    }
}

const fn max_stage(value: crate::program::expression::Stage) -> u8 {
    match value {
        crate::program::expression::Stage::Build => 0x00,
        crate::program::expression::Stage::Temporal => 0x01,
        crate::program::expression::Stage::Const => unreachable!(),
    }
}

fn temporal_lowering(digest: &mut Sha256, value: Option<TemporalLoweringOpcode>) {
    let bytes = match value {
        None => &[0x00][..],
        Some(TemporalLoweringOpcode::ComposeVector) => &[0x01, 0x00],
        Some(TemporalLoweringOpcode::ComposePoint) => &[0x01, 0x01],
        Some(TemporalLoweringOpcode::ComposeRect) => &[0x01, 0x02],
    };
    digest.update(bytes);
}

fn framed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
#[path = "digest/tests.rs"]
mod tests;
