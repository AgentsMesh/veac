use super::{EffectKind, EffectParameter};

impl EffectKind {
    pub const BUILT_IN: [Self; 10] = [
        Self::VideoColorAdjust,
        Self::VideoBlur,
        Self::VideoSharpen,
        Self::VideoVignette,
        Self::VideoGrain,
        Self::VideoChromaKey,
        Self::VideoLumaKey,
        Self::VideoChromaSpill,
        Self::VideoStabilize,
        Self::AudioNormalize,
    ];
    pub const ALL: [Self; 11] = [
        Self::VideoColorAdjust,
        Self::VideoBlur,
        Self::VideoSharpen,
        Self::VideoVignette,
        Self::VideoGrain,
        Self::VideoChromaKey,
        Self::VideoLumaKey,
        Self::VideoChromaSpill,
        Self::VideoStabilize,
        Self::AudioNormalize,
        Self::VideoPluginReferenceMonochromeV1,
    ];

    pub const fn type_name(self) -> &'static str {
        match self {
            Self::VideoColorAdjust => "video.color_adjust",
            Self::VideoBlur => "video.blur",
            Self::VideoSharpen => "video.sharpen",
            Self::VideoVignette => "video.vignette",
            Self::VideoGrain => "video.grain",
            Self::VideoChromaKey => "video.chroma_key",
            Self::VideoLumaKey => "video.luma_key",
            Self::VideoChromaSpill => "video.chroma_spill",
            Self::VideoStabilize => "video.stabilize",
            Self::AudioNormalize => "audio.normalize",
            Self::VideoPluginReferenceMonochromeV1 => crate::REFERENCE_MONOCHROME_EFFECT_TYPE,
        }
    }

    pub fn from_type_name(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.type_name() == value)
    }
}

impl EffectParameter {
    pub const ALL: [Self; 15] = [
        Self::Brightness,
        Self::Contrast,
        Self::Saturation,
        Self::Radius,
        Self::Amount,
        Self::Color,
        Self::Similarity,
        Self::Blend,
        Self::Threshold,
        Self::Tolerance,
        Self::Softness,
        Self::Invert,
        Self::Range,
        Self::Enabled,
        Self::TargetLufs,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Brightness => "brightness",
            Self::Contrast => "contrast",
            Self::Saturation => "saturation",
            Self::Radius => "radius",
            Self::Amount => "amount",
            Self::Color => "color",
            Self::Similarity => "similarity",
            Self::Blend => "blend",
            Self::Threshold => "threshold",
            Self::Tolerance => "tolerance",
            Self::Softness => "softness",
            Self::Invert => "invert",
            Self::Range => "range",
            Self::Enabled => "enabled",
            Self::TargetLufs => "target_lufs",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|parameter| parameter.name() == value)
    }
}
