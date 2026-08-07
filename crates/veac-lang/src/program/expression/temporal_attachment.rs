use super::{FunctionEffect, PrimitiveType, ValueType};
use crate::program::DomainType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TemporalOwnerKind {
    Item,
    Apply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TemporalTargetKind {
    Clip,
    Text,
    ClipMask,
    ClipEffect,
    Apply,
    ApplyMask,
    ApplyEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TemporalAttachmentKind {
    ClipVisualPosition,
    ClipVisualScale,
    ClipVisualRotation,
    ClipVisualCrop,
    ClipVisualOpacity,
    ClipAudioGain,
    ClipAudioPan,
    ClipMaskPosition,
    ClipMaskScale,
    ClipMaskRotation,
    ClipMaskFeather,
    ClipMaskExpansion,
    ClipTextPosition,
    ClipTextScale,
    ClipTextRotation,
    ClipTextReveal,
    ClipTextHighlightProgress,
    ClipTextOpacity,
    ClipEffectParameter,
    ApplyOpacity,
    ApplyMaskPosition,
    ApplyMaskScale,
    ApplyMaskRotation,
    ApplyMaskFeather,
    ApplyMaskExpansion,
    ApplyEffectParameter,
}

impl TemporalAttachmentKind {
    pub(crate) const fn owner(self) -> TemporalOwnerKind {
        use TemporalAttachmentKind::*;
        match self {
            ApplyOpacity | ApplyMaskPosition | ApplyMaskScale | ApplyMaskRotation
            | ApplyMaskFeather | ApplyMaskExpansion | ApplyEffectParameter => {
                TemporalOwnerKind::Apply
            }
            _ => TemporalOwnerKind::Item,
        }
    }

    pub(crate) const fn selector_types(self) -> &'static [PrimitiveType] {
        use PrimitiveType::{Identifier, Integer};
        use TemporalAttachmentKind::*;
        match self {
            ClipMaskPosition | ClipMaskScale | ClipMaskRotation | ClipMaskFeather
            | ClipMaskExpansion | ApplyMaskPosition | ApplyMaskScale | ApplyMaskRotation
            | ApplyMaskFeather | ApplyMaskExpansion => &[Integer],
            ClipEffectParameter => &[Identifier, Identifier],
            ApplyEffectParameter => &[Identifier, Identifier, Identifier],
            _ => &[],
        }
    }

    pub(crate) fn result_type(self) -> ValueType {
        use TemporalAttachmentKind::*;
        match self {
            ClipVisualPosition | ClipTextPosition => ValueType::domain(DomainType::Point),
            ClipVisualScale | ClipMaskPosition | ClipMaskScale | ClipTextScale
            | ApplyMaskPosition | ApplyMaskScale => ValueType::domain(DomainType::Vector),
            ClipVisualCrop => ValueType::domain(DomainType::Rect),
            ClipVisualRotation | ClipMaskRotation | ClipTextRotation | ApplyMaskRotation => {
                PrimitiveType::Angle.into()
            }
            _ => PrimitiveType::Scalar.into(),
        }
    }

    pub(crate) fn owner_type(self) -> ValueType {
        ValueType::domain(match self.owner() {
            TemporalOwnerKind::Item => DomainType::Item,
            TemporalOwnerKind::Apply => DomainType::Apply,
        })
    }

    pub(crate) fn animation_type(self) -> ValueType {
        let mut parameters = vec![PrimitiveType::Time.into()];
        if self.owner() == TemporalOwnerKind::Item {
            parameters.extend([
                PrimitiveType::Time.into(),
                PrimitiveType::Integer.into(),
                PrimitiveType::Scalar.into(),
                PrimitiveType::Time.into(),
            ]);
        } else {
            parameters.push(PrimitiveType::Integer.into());
        }
        ValueType::function(parameters, self.result_type(), FunctionEffect::Pure)
            .expect("closed temporal animation signature")
    }
}

mod catalog;

impl TemporalTargetKind {
    pub(crate) const ALL: [Self; 7] = [
        Self::Clip,
        Self::Text,
        Self::ClipMask,
        Self::ClipEffect,
        Self::Apply,
        Self::ApplyMask,
        Self::ApplyEffect,
    ];

    pub(crate) fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.source_name() == value)
    }

    pub(crate) const fn source_name(self) -> &'static str {
        match self {
            Self::Clip => "clip",
            Self::Text => "text",
            Self::ClipMask => "clip-mask",
            Self::ClipEffect => "clip-effect",
            Self::Apply => "apply",
            Self::ApplyMask => "apply-mask",
            Self::ApplyEffect => "apply-effect",
        }
    }
}
