use super::{TemporalAttachmentKind, TemporalTargetKind};
use crate::program::model::TemporalProperty;

impl TemporalAttachmentKind {
    pub(crate) const fn opcode(self) -> u8 {
        self as u8
    }

    pub(crate) fn from_opcode(value: u8) -> Option<Self> {
        ALL.into_iter().find(|kind| kind.opcode() == value)
    }

    pub(crate) const fn from_parts(
        property: TemporalProperty,
        target: TemporalTargetKind,
    ) -> Option<Self> {
        use TemporalAttachmentKind as K;
        use TemporalProperty as P;
        use TemporalTargetKind as T;
        Some(match (target, property) {
            (T::Clip, P::VisualPosition) => K::ClipVisualPosition,
            (T::Clip, P::VisualScale) => K::ClipVisualScale,
            (T::Clip, P::VisualRotation) => K::ClipVisualRotation,
            (T::Clip, P::VisualCrop) => K::ClipVisualCrop,
            (T::Clip, P::VisualOpacity) => K::ClipVisualOpacity,
            (T::Clip, P::AudioGain) => K::ClipAudioGain,
            (T::Clip, P::AudioPan) => K::ClipAudioPan,
            (T::ClipMask, P::MaskPosition) => K::ClipMaskPosition,
            (T::ClipMask, P::MaskScale) => K::ClipMaskScale,
            (T::ClipMask, P::MaskRotation) => K::ClipMaskRotation,
            (T::ClipMask, P::MaskFeather) => K::ClipMaskFeather,
            (T::ClipMask, P::MaskExpansion) => K::ClipMaskExpansion,
            (T::Text, P::TextPosition) => K::ClipTextPosition,
            (T::Text, P::TextScale) => K::ClipTextScale,
            (T::Text, P::TextRotation) => K::ClipTextRotation,
            (T::Text, P::TextReveal) => K::ClipTextReveal,
            (T::Text, P::TextHighlightProgress) => K::ClipTextHighlightProgress,
            (T::Text, P::TextOpacity) => K::ClipTextOpacity,
            (T::ClipEffect, P::EffectParameter) => K::ClipEffectParameter,
            (T::Apply, P::ApplyOpacity) => K::ApplyOpacity,
            (T::ApplyMask, P::MaskPosition) => K::ApplyMaskPosition,
            (T::ApplyMask, P::MaskScale) => K::ApplyMaskScale,
            (T::ApplyMask, P::MaskRotation) => K::ApplyMaskRotation,
            (T::ApplyMask, P::MaskFeather) => K::ApplyMaskFeather,
            (T::ApplyMask, P::MaskExpansion) => K::ApplyMaskExpansion,
            (T::ApplyEffect, P::EffectParameter) => K::ApplyEffectParameter,
            _ => return None,
        })
    }

    pub(crate) fn parts(self) -> (TemporalProperty, TemporalTargetKind) {
        for target in TemporalTargetKind::ALL {
            for property in TemporalProperty::ALL {
                if Self::from_parts(property, target) == Some(self) {
                    return (property, target);
                }
            }
        }
        unreachable!("closed temporal attachment kind")
    }
}

const ALL: [TemporalAttachmentKind; 26] = [
    TemporalAttachmentKind::ClipVisualPosition,
    TemporalAttachmentKind::ClipVisualScale,
    TemporalAttachmentKind::ClipVisualRotation,
    TemporalAttachmentKind::ClipVisualCrop,
    TemporalAttachmentKind::ClipVisualOpacity,
    TemporalAttachmentKind::ClipAudioGain,
    TemporalAttachmentKind::ClipAudioPan,
    TemporalAttachmentKind::ClipMaskPosition,
    TemporalAttachmentKind::ClipMaskScale,
    TemporalAttachmentKind::ClipMaskRotation,
    TemporalAttachmentKind::ClipMaskFeather,
    TemporalAttachmentKind::ClipMaskExpansion,
    TemporalAttachmentKind::ClipTextPosition,
    TemporalAttachmentKind::ClipTextScale,
    TemporalAttachmentKind::ClipTextRotation,
    TemporalAttachmentKind::ClipTextReveal,
    TemporalAttachmentKind::ClipTextHighlightProgress,
    TemporalAttachmentKind::ClipTextOpacity,
    TemporalAttachmentKind::ClipEffectParameter,
    TemporalAttachmentKind::ApplyOpacity,
    TemporalAttachmentKind::ApplyMaskPosition,
    TemporalAttachmentKind::ApplyMaskScale,
    TemporalAttachmentKind::ApplyMaskRotation,
    TemporalAttachmentKind::ApplyMaskFeather,
    TemporalAttachmentKind::ApplyMaskExpansion,
    TemporalAttachmentKind::ApplyEffectParameter,
];
