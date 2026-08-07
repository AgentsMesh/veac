#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum TemporalProperty {
    VisualPosition,
    VisualScale,
    VisualRotation,
    VisualCrop,
    VisualOpacity,
    AudioGain,
    AudioPan,
    MaskPosition,
    MaskScale,
    MaskRotation,
    MaskFeather,
    MaskExpansion,
    TextPosition,
    TextScale,
    TextRotation,
    TextReveal,
    TextHighlightProgress,
    TextOpacity,
    EffectParameter,
    ApplyOpacity,
}

impl TemporalProperty {
    pub(crate) const ALL: [Self; 20] = [
        Self::VisualPosition,
        Self::VisualScale,
        Self::VisualRotation,
        Self::VisualCrop,
        Self::VisualOpacity,
        Self::AudioGain,
        Self::AudioPan,
        Self::MaskPosition,
        Self::MaskScale,
        Self::MaskRotation,
        Self::MaskFeather,
        Self::MaskExpansion,
        Self::TextPosition,
        Self::TextScale,
        Self::TextRotation,
        Self::TextReveal,
        Self::TextHighlightProgress,
        Self::TextOpacity,
        Self::EffectParameter,
        Self::ApplyOpacity,
    ];

    pub(crate) fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|property| property.source_name() == value)
    }

    pub(crate) const fn source_name(self) -> &'static str {
        match self {
            Self::VisualPosition => "visual-position",
            Self::VisualScale => "visual-scale",
            Self::VisualRotation => "visual-rotation",
            Self::VisualCrop => "visual-crop",
            Self::VisualOpacity => "visual-opacity",
            Self::AudioGain => "audio-gain",
            Self::AudioPan => "audio-pan",
            Self::MaskPosition => "mask-position",
            Self::MaskScale => "mask-scale",
            Self::MaskRotation => "mask-rotation",
            Self::MaskFeather => "mask-feather",
            Self::MaskExpansion => "mask-expansion",
            Self::TextPosition => "text-position",
            Self::TextScale => "text-scale",
            Self::TextRotation => "text-rotation",
            Self::TextReveal => "text-reveal",
            Self::TextHighlightProgress => "text-highlight-progress",
            Self::TextOpacity => "text-opacity",
            Self::EffectParameter => "effect-parameter",
            Self::ApplyOpacity => "apply-opacity",
        }
    }
}
