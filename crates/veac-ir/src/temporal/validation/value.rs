use crate::MAX_SAFE_INTEGER;

use super::Validator;
use crate::temporal::{TemporalCurvePosition, TemporalValue, MAX_TEMPORAL_TEXT_BYTES};

impl Validator {
    pub(super) fn value(&mut self, value: &TemporalValue, pointer: &str) {
        self.charge_value(value.logical_bytes(), pointer);
        let valid = match value {
            TemporalValue::Boolean { .. } | TemporalValue::Color { .. } => true,
            TemporalValue::Integer { value } => value.unsigned_abs() <= MAX_SAFE_INTEGER,
            TemporalValue::Scalar { value } | TemporalValue::Angle { degrees: value } => {
                canonical_float(*value)
            }
            TemporalValue::Time { value } => value.is_valid(),
            TemporalValue::Length { value } => canonical_float(value.value),
            TemporalValue::Vec2 { value } => canonical_float(value.x) && canonical_float(value.y),
            TemporalValue::Point { value } => {
                canonical_float(value.x.value) && canonical_float(value.y.value)
            }
            TemporalValue::Rect { value } => [value.x, value.y, value.width, value.height]
                .into_iter()
                .all(canonical_float),
            TemporalValue::Text { value } => value.len() <= MAX_TEMPORAL_TEXT_BYTES,
        };
        if !valid {
            self.push(
                "TEMPORAL_VALUE",
                pointer,
                "value is outside the temporal value contract",
            );
        }
    }

    pub(super) fn position(&mut self, value: TemporalCurvePosition, pointer: &str) -> bool {
        let valid = match value {
            TemporalCurvePosition::Scalar { value } => canonical_float(value),
            TemporalCurvePosition::Time { value } => value.is_valid(),
        };
        if !valid {
            self.push(
                "TEMPORAL_CURVE_POSITION",
                pointer,
                "curve position is invalid",
            );
        }
        valid
    }
}

pub(super) fn canonical_float(value: f64) -> bool {
    value.is_finite() && !(value == 0.0 && value.is_sign_negative())
}
