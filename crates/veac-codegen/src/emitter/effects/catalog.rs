use veac_plan::canonical::{ParameterValue, ParameterValueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::emitter) enum EffectKind {
    VideoColorAdjust,
    VideoBlur,
    VideoSharpen,
    VideoVignette,
    VideoGrain,
    VideoChromaKey,
    VideoLumaKey,
    VideoChromaSpill,
    VideoStabilize,
    AudioNormalize,
}

impl EffectKind {
    pub(super) fn replaces_alpha(self) -> bool {
        matches!(self, Self::VideoChromaKey | Self::VideoLumaKey)
    }
}

const EFFECTS: &[(&str, EffectKind)] = &[
    ("video.color_adjust", EffectKind::VideoColorAdjust),
    ("video.blur", EffectKind::VideoBlur),
    ("video.sharpen", EffectKind::VideoSharpen),
    ("video.vignette", EffectKind::VideoVignette),
    ("video.grain", EffectKind::VideoGrain),
    ("video.chroma_key", EffectKind::VideoChromaKey),
    ("video.luma_key", EffectKind::VideoLumaKey),
    ("video.chroma_spill", EffectKind::VideoChromaSpill),
    ("video.stabilize", EffectKind::VideoStabilize),
    ("audio.normalize", EffectKind::AudioNormalize),
];
const PARAMETER_KINDS: &[ParameterValueKind] = &[
    ParameterValueKind::Number,
    ParameterValueKind::NumberCurve,
    ParameterValueKind::Boolean,
    ParameterValueKind::Color,
];

pub(in crate::emitter) fn effect_kind(effect_type: &str) -> Option<EffectKind> {
    EFFECTS
        .iter()
        .find_map(|(name, kind)| (*name == effect_type).then_some(*kind))
}

pub(in crate::emitter) fn supports_parameter(value: &ParameterValue) -> bool {
    PARAMETER_KINDS.contains(&value.kind())
}

#[cfg(test)]
mod tests;
