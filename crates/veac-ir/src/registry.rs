mod validation;

use serde::Serialize;

use crate::{Effect, EffectKind, EffectParameter, EffectParameterRef};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    Number,
    Boolean,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ParameterSpec {
    pub parameter: EffectParameter,
    pub value_type: ParameterType,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub supports_curve: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct EffectSpec {
    pub kind: EffectKind,
    pub effect_type: &'static str,
    pub parameters: &'static [ParameterSpec],
}

const COLOR_ADJUST: &[ParameterSpec] = &[
    curve(EffectParameter::Brightness, -1.0, 1.0),
    curve(EffectParameter::Contrast, 0.0, 4.0),
    curve(EffectParameter::Saturation, 0.0, 4.0),
];
const BLUR: &[ParameterSpec] = &[curve(EffectParameter::Radius, 0.0, 100.0)];
const DIRECTIONAL_BLUR: &[ParameterSpec] = &[
    curve(EffectParameter::AngleDegrees, 0.0, 360.0),
    curve(EffectParameter::Radius, 0.0, 100.0),
];
const SHARPEN: &[ParameterSpec] = &[curve(EffectParameter::Amount, 0.0, 10.0)];
const UNIT_AMOUNT: &[ParameterSpec] = &[curve(EffectParameter::Amount, 0.0, 1.0)];
const CHROMA_KEY: &[ParameterSpec] = &[
    color(EffectParameter::Color),
    curve(EffectParameter::Similarity, 0.00001, 1.0),
    curve(EffectParameter::Blend, 0.0, 1.0),
];
const LUMA_KEY: &[ParameterSpec] = &[
    curve(EffectParameter::Threshold, 0.0, 1.0),
    curve(EffectParameter::Tolerance, 0.0, 1.0),
    curve(EffectParameter::Softness, 0.0, 1.0),
    boolean(EffectParameter::Invert),
];
const CHROMA_SPILL: &[ParameterSpec] = &[
    color(EffectParameter::Color),
    curve(EffectParameter::Amount, 0.0, 1.0),
    curve(EffectParameter::Range, 0.0, 1.0),
];
const STABILIZE: &[ParameterSpec] = &[boolean(EffectParameter::Enabled)];
const NORMALIZE: &[ParameterSpec] = &[number(EffectParameter::TargetLufs, -70.0, -5.0)];

const EFFECTS: &[EffectSpec] = &[
    spec(EffectKind::VideoColorAdjust, COLOR_ADJUST),
    spec(EffectKind::VideoBlur, BLUR),
    spec(EffectKind::VideoDirectionalBlur, DIRECTIONAL_BLUR),
    spec(EffectKind::VideoSharpen, SHARPEN),
    spec(EffectKind::VideoVignette, UNIT_AMOUNT),
    spec(EffectKind::VideoGrain, UNIT_AMOUNT),
    spec(EffectKind::VideoChromaKey, CHROMA_KEY),
    spec(EffectKind::VideoLumaKey, LUMA_KEY),
    spec(EffectKind::VideoChromaSpill, CHROMA_SPILL),
    spec(EffectKind::VideoStabilize, STABILIZE),
    spec(EffectKind::AudioNormalize, NORMALIZE),
];

const fn spec(kind: EffectKind, parameters: &'static [ParameterSpec]) -> EffectSpec {
    EffectSpec {
        kind,
        effect_type: kind.type_name(),
        parameters,
    }
}

const fn curve(parameter: EffectParameter, minimum: f64, maximum: f64) -> ParameterSpec {
    ParameterSpec {
        parameter,
        value_type: ParameterType::Number,
        minimum: Some(minimum),
        maximum: Some(maximum),
        supports_curve: true,
    }
}

const fn number(parameter: EffectParameter, minimum: f64, maximum: f64) -> ParameterSpec {
    ParameterSpec {
        supports_curve: false,
        ..curve(parameter, minimum, maximum)
    }
}

const fn boolean(parameter: EffectParameter) -> ParameterSpec {
    typed(parameter, ParameterType::Boolean)
}

const fn color(parameter: EffectParameter) -> ParameterSpec {
    typed(parameter, ParameterType::Color)
}

const fn typed(parameter: EffectParameter, value_type: ParameterType) -> ParameterSpec {
    ParameterSpec {
        parameter,
        value_type,
        minimum: None,
        maximum: None,
        supports_curve: false,
    }
}

pub fn built_in_effect(kind: EffectKind) -> Option<EffectSpec> {
    EFFECTS.iter().copied().find(|effect| effect.kind == kind)
}

pub fn built_in_effect_type(effect_type: &str) -> Option<EffectSpec> {
    EffectKind::from_type_name(effect_type).and_then(built_in_effect)
}

pub fn built_in_effects() -> &'static [EffectSpec] {
    EFFECTS
}

pub fn parameter_matches_effect(spec: ParameterSpec, effect: &Effect) -> bool {
    effect
        .parameter(spec.parameter)
        .is_some_and(|value| parameter_matches(spec, value))
}

pub fn parameter_matches(spec: ParameterSpec, value: EffectParameterRef<'_>) -> bool {
    validation::matches(spec, value)
}
