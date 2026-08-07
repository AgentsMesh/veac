use crate::{Animatable, EffectParameterRef, ParameterSpec, ParameterType};

pub(super) fn matches(spec: ParameterSpec, value: EffectParameterRef<'_>) -> bool {
    match (spec.value_type, value) {
        (ParameterType::Number, EffectParameterRef::Number(value)) if !spec.supports_curve => {
            in_range(spec, value)
        }
        (ParameterType::Number, EffectParameterRef::Curve(value)) if spec.supports_curve => {
            curve_in_range(spec, value)
        }
        (ParameterType::Boolean, EffectParameterRef::Boolean(_))
        | (ParameterType::Color, EffectParameterRef::Color(_)) => true,
        _ => false,
    }
}

fn curve_in_range(spec: ParameterSpec, value: &Animatable<f64>) -> bool {
    let keyframes = match value {
        Animatable::Binding { .. } => return true,
        Animatable::Constant { value } => return in_range(spec, *value),
        Animatable::Keyframes { keyframes } => keyframes,
    };
    keyframes.iter().all(|key| in_range(spec, key.value))
        && keyframes.windows(2).all(|pair| {
            pair[0]
                .interpolation
                .intermediate_extrema()
                .is_none_or(|amounts| {
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
