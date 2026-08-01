use crate::ParameterValue;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
    Number,
    Boolean,
    Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParameterSpec {
    pub name: &'static str,
    pub value_type: ParameterType,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub supports_curve: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectSpec {
    pub effect_type: &'static str,
    pub parameters: &'static [ParameterSpec],
}

const COLOR_ADJUST: &[ParameterSpec] = &[
    number("brightness", -1.0, 1.0),
    number("contrast", 0.0, 4.0),
    number("saturation", 0.0, 4.0),
];
const BLUR: &[ParameterSpec] = &[number("radius", 0.0, 100.0)];
const SHARPEN: &[ParameterSpec] = &[number("amount", 0.0, 10.0)];
const VIGNETTE: &[ParameterSpec] = &[number("amount", 0.0, 1.0)];
const GRAIN: &[ParameterSpec] = &[number("amount", 0.0, 1.0)];
const CHROMA_KEY: &[ParameterSpec] = &[
    ParameterSpec {
        name: "color",
        value_type: ParameterType::Color,
        minimum: None,
        maximum: None,
        supports_curve: false,
    },
    number("similarity", 0.00001, 1.0),
    number("blend", 0.0, 1.0),
];
const LUMA_KEY: &[ParameterSpec] = &[
    number("threshold", 0.0, 1.0),
    number("tolerance", 0.0, 1.0),
    number("softness", 0.0, 1.0),
    boolean("invert"),
];
const CHROMA_SPILL: &[ParameterSpec] = &[
    ParameterSpec {
        name: "color",
        value_type: ParameterType::Color,
        minimum: None,
        maximum: None,
        supports_curve: false,
    },
    number("amount", 0.0, 1.0),
    number("range", 0.0, 1.0),
];
const STABILIZE: &[ParameterSpec] = &[ParameterSpec {
    name: "enabled",
    value_type: ParameterType::Boolean,
    minimum: None,
    maximum: None,
    supports_curve: false,
}];
const NORMALIZE: &[ParameterSpec] = &[static_number("target_lufs", -70.0, -5.0)];
const EFFECTS: &[EffectSpec] = &[
    EffectSpec {
        effect_type: "video.color_adjust",
        parameters: COLOR_ADJUST,
    },
    EffectSpec {
        effect_type: "video.blur",
        parameters: BLUR,
    },
    EffectSpec {
        effect_type: "video.sharpen",
        parameters: SHARPEN,
    },
    EffectSpec {
        effect_type: "video.vignette",
        parameters: VIGNETTE,
    },
    EffectSpec {
        effect_type: "video.grain",
        parameters: GRAIN,
    },
    EffectSpec {
        effect_type: "video.chroma_key",
        parameters: CHROMA_KEY,
    },
    EffectSpec {
        effect_type: "video.luma_key",
        parameters: LUMA_KEY,
    },
    EffectSpec {
        effect_type: "video.chroma_spill",
        parameters: CHROMA_SPILL,
    },
    EffectSpec {
        effect_type: "video.stabilize",
        parameters: STABILIZE,
    },
    EffectSpec {
        effect_type: "audio.normalize",
        parameters: NORMALIZE,
    },
];
const fn number(name: &'static str, minimum: f64, maximum: f64) -> ParameterSpec {
    ParameterSpec {
        name,
        value_type: ParameterType::Number,
        minimum: Some(minimum),
        maximum: Some(maximum),
        supports_curve: true,
    }
}

const fn static_number(name: &'static str, minimum: f64, maximum: f64) -> ParameterSpec {
    ParameterSpec {
        supports_curve: false,
        ..number(name, minimum, maximum)
    }
}

const fn boolean(name: &'static str) -> ParameterSpec {
    ParameterSpec {
        name,
        value_type: ParameterType::Boolean,
        minimum: None,
        maximum: None,
        supports_curve: false,
    }
}

pub fn built_in_effect(effect_type: &str) -> Option<EffectSpec> {
    EFFECTS
        .iter()
        .copied()
        .find(|effect| effect.effect_type == effect_type)
}

pub fn built_in_effects() -> &'static [EffectSpec] {
    EFFECTS
}

pub fn parameter_matches(spec: ParameterSpec, value: &ParameterValue) -> bool {
    match (spec.value_type, value) {
        (ParameterType::Number, ParameterValue::Number { value }) => in_range(spec, *value),
        (ParameterType::Number, ParameterValue::NumberCurve { value }) if spec.supports_curve => {
            curve_in_range(spec, value)
        }
        (ParameterType::Boolean, ParameterValue::Boolean { .. })
        | (ParameterType::Color, ParameterValue::Color { .. }) => true,
        _ => false,
    }
}

fn curve_in_range(spec: ParameterSpec, value: &crate::Animatable<f64>) -> bool {
    let crate::Animatable::Keyframes { keyframes } = value else {
        let crate::Animatable::Constant { value } = value else {
            unreachable!()
        };
        return in_range(spec, *value);
    };
    keyframes.iter().all(|key| in_range(spec, key.value))
        && keyframes.windows(2).all(|pair| {
            if !matches!(pair[0].interpolation, crate::Interpolation::Spring { .. }) {
                return true;
            }
            pair[0]
                .interpolation
                .spring_extrema()
                .is_some_and(|amounts| {
                    amounts.into_iter().all(|amount| {
                        in_range(
                            spec,
                            pair[0].value + (pair[1].value - pair[0].value) * amount,
                        )
                    })
                })
        })
}

fn in_range(spec: ParameterSpec, value: f64) -> bool {
    value.is_finite()
        && spec.minimum.is_none_or(|minimum| value >= minimum)
        && spec.maximum.is_none_or(|maximum| value <= maximum)
}
