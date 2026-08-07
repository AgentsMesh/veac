use std::cmp::Ordering;

use veac_ir::{ApplyId, ApplyStageId, EffectId, EffectParameter, ItemId, SequenceId, TemporalType};

use super::{ClipTemporalProperty, MaskTemporalProperty, TextTemporalProperty};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ExecutableTemporalSink {
    Clip {
        item_id: ItemId,
        property: ClipTemporalProperty,
    },
    ClipMask {
        item_id: ItemId,
        mask_index: u32,
        property: MaskTemporalProperty,
    },
    ClipText {
        item_id: ItemId,
        property: TextTemporalProperty,
    },
    ClipEffect {
        item_id: ItemId,
        effect_id: EffectId,
        parameter: EffectParameter,
    },
    ApplyOpacity {
        sequence_id: SequenceId,
        apply_id: ApplyId,
    },
    ApplyMask {
        sequence_id: SequenceId,
        apply_id: ApplyId,
        mask_index: u32,
        property: MaskTemporalProperty,
    },
    ApplyEffect {
        sequence_id: SequenceId,
        apply_id: ApplyId,
        stage_id: ApplyStageId,
        effect_id: EffectId,
        parameter: EffectParameter,
    },
}

impl ExecutableTemporalSink {
    pub(crate) fn clip(item_id: ItemId, property: ClipTemporalProperty) -> Self {
        Self::Clip { item_id, property }
    }

    pub(crate) fn expected_type(&self) -> TemporalType {
        match self {
            Self::Clip { property, .. } => property.expected_type(),
            Self::ClipMask { property, .. } | Self::ApplyMask { property, .. } => {
                property.expected_type()
            }
            Self::ClipText { property, .. } => property.expected_type(),
            Self::ClipEffect { .. } | Self::ApplyOpacity { .. } | Self::ApplyEffect { .. } => {
                TemporalType::Scalar
            }
        }
    }

    pub(in crate::program::executable) fn item_id(&self) -> Option<&ItemId> {
        match self {
            Self::Clip { item_id, .. }
            | Self::ClipMask { item_id, .. }
            | Self::ClipText { item_id, .. }
            | Self::ClipEffect { item_id, .. } => Some(item_id),
            Self::ApplyOpacity { .. } | Self::ApplyMask { .. } | Self::ApplyEffect { .. } => None,
        }
    }

    pub(in crate::program::executable) fn sequence_id(&self) -> Option<&SequenceId> {
        match self {
            Self::ApplyOpacity { sequence_id, .. }
            | Self::ApplyMask { sequence_id, .. }
            | Self::ApplyEffect { sequence_id, .. } => Some(sequence_id),
            Self::Clip { .. }
            | Self::ClipMask { .. }
            | Self::ClipText { .. }
            | Self::ClipEffect { .. } => None,
        }
    }

    pub(in crate::program::executable) fn identity_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Self::Clip { item_id, property } => {
                bytes.extend_from_slice(&[0x01, property.opcode()]);
                frame(&mut bytes, item_id.as_str());
            }
            Self::ClipMask {
                item_id,
                mask_index,
                property,
            } => {
                bytes.extend_from_slice(&[0x02, property.opcode()]);
                frame(&mut bytes, item_id.as_str());
                bytes.extend_from_slice(&mask_index.to_be_bytes());
            }
            Self::ClipText { item_id, property } => {
                bytes.extend_from_slice(&[0x03, property.opcode()]);
                frame(&mut bytes, item_id.as_str());
            }
            Self::ClipEffect {
                item_id,
                effect_id,
                parameter,
            } => {
                bytes.push(0x04);
                frame(&mut bytes, item_id.as_str());
                frame(&mut bytes, effect_id.as_str());
                frame(&mut bytes, parameter.name());
            }
            Self::ApplyOpacity {
                sequence_id,
                apply_id,
            } => {
                bytes.push(0x05);
                frame(&mut bytes, sequence_id.as_str());
                frame(&mut bytes, apply_id.as_str());
            }
            Self::ApplyMask {
                sequence_id,
                apply_id,
                mask_index,
                property,
            } => {
                bytes.extend_from_slice(&[0x06, property.opcode()]);
                frame(&mut bytes, sequence_id.as_str());
                frame(&mut bytes, apply_id.as_str());
                bytes.extend_from_slice(&mask_index.to_be_bytes());
            }
            Self::ApplyEffect {
                sequence_id,
                apply_id,
                stage_id,
                effect_id,
                parameter,
            } => {
                bytes.push(0x07);
                frame(&mut bytes, sequence_id.as_str());
                frame(&mut bytes, apply_id.as_str());
                frame(&mut bytes, stage_id.as_str());
                frame(&mut bytes, effect_id.as_str());
                frame(&mut bytes, parameter.name());
            }
        }
        bytes
    }
}

impl Ord for ExecutableTemporalSink {
    fn cmp(&self, other: &Self) -> Ordering {
        self.identity_bytes().cmp(&other.identity_bytes())
    }
}

impl PartialOrd for ExecutableTemporalSink {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn frame(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend_from_slice(&(value.len() as u64).to_be_bytes());
    bytes.extend_from_slice(value.as_bytes());
}
