use super::super::super::super::super::ResidualizationError;
use super::error;
use super::keys::key_error;
use veac_ir::{
    Interpolation, TemporalCurveKey, TemporalCurvePosition, TemporalType, MAX_TEMPORAL_CURVE_KEYS,
};

pub(super) fn keys(
    input: TemporalType,
    keys: &[TemporalCurveKey],
    span: std::ops::Range<usize>,
) -> Result<(), ResidualizationError> {
    if keys.is_empty() || keys.len() > MAX_TEMPORAL_CURVE_KEYS {
        return Err(key_error(span));
    }
    let value_type = keys[0].value.value_type();
    let valid = keys
        .iter()
        .all(|key| key.position.value_type() == input && key.value.value_type() == value_type)
        && keys
            .windows(2)
            .all(|pair| position_before(pair[0].position, pair[1].position));
    valid.then_some(()).ok_or_else(|| {
        error(
            "RESIDUAL_CURVE_KEY_CONTRACT",
            "curve keys must be type-consistent and strictly increasing",
            span,
        )
    })
}

fn position_before(left: TemporalCurvePosition, right: TemporalCurvePosition) -> bool {
    match (left, right) {
        (
            TemporalCurvePosition::Scalar { value: a },
            TemporalCurvePosition::Scalar { value: b },
        ) => a < b,
        (TemporalCurvePosition::Time { value: a }, TemporalCurvePosition::Time { value: b }) => {
            a < b
        }
        _ => false,
    }
}

pub(super) fn interpolation(value: &Interpolation) -> bool {
    match value {
        Interpolation::Spring { .. } => value.spring_coefficients().is_some(),
        Interpolation::CubicBezier { x1, y1, x2, y2 } => [x1, y1, x2, y2]
            .into_iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value)),
        _ => true,
    }
}
