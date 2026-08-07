use crate::temporal::{
    TemporalCurveKey, TemporalCurvePosition, TemporalType, TemporalValue, MAX_TEMPORAL_CURVE_KEYS,
};
use crate::Interpolation;

use super::{operation, value::canonical_float, Validator};

impl Validator {
    pub(super) fn curve(
        &mut self,
        input: TemporalType,
        keys: &[TemporalCurveKey],
        pointer: &str,
    ) -> Option<TemporalType> {
        if keys.is_empty() || keys.len() > MAX_TEMPORAL_CURVE_KEYS {
            self.push(
                "TEMPORAL_CURVE_KEYS",
                format!("{pointer}/keys"),
                "curve key count is outside the contract",
            );
            return None;
        }
        let output = keys[0].value.value_type();
        for (index, key) in keys.iter().enumerate() {
            let path = format!("{pointer}/keys/{index}");
            self.position(key.position, &format!("{path}/position"));
            self.value(&key.value, &format!("{path}/value"));
            if key.position.value_type() != input
                || key.value.value_type() != output
                || !interpolation_valid(&key.interpolation)
            {
                self.push(
                    "TEMPORAL_CURVE_TYPE",
                    &path,
                    "curve key type or interpolation is invalid",
                );
            }
            if !curve_value_compatible(&keys[0].value, &key.value) {
                self.push(
                    "TEMPORAL_CURVE_VALUE",
                    &path,
                    "curve values have incompatible units",
                );
            }
            if index + 1 < keys.len()
                && !matches!(key.interpolation, Interpolation::Hold)
                && !operation::interpolatable(output)
            {
                self.push(
                    "TEMPORAL_CURVE_INTERPOLATION",
                    path,
                    "value type supports hold interpolation only",
                );
            }
        }
        if keys
            .windows(2)
            .any(|pair| !position_before(pair[0].position, pair[1].position))
        {
            self.push(
                "TEMPORAL_CURVE_ORDER",
                format!("{pointer}/keys"),
                "curve positions must be strictly increasing with one type",
            );
        }
        Some(output)
    }
}

fn position_before(left: TemporalCurvePosition, right: TemporalCurvePosition) -> bool {
    match (left, right) {
        (
            TemporalCurvePosition::Scalar { value: left },
            TemporalCurvePosition::Scalar { value: right },
        ) => left < right,
        (
            TemporalCurvePosition::Time { value: left },
            TemporalCurvePosition::Time { value: right },
        ) => left < right,
        _ => false,
    }
}

fn interpolation_valid(value: &Interpolation) -> bool {
    match value {
        Interpolation::CubicBezier { x1, y1, x2, y2 } => {
            [*x1, *y1, *x2, *y2].into_iter().all(canonical_float)
                && (0.0..=1.0).contains(x1)
                && (0.0..=1.0).contains(y1)
                && (0.0..=1.0).contains(x2)
                && (0.0..=1.0).contains(y2)
        }
        Interpolation::Spring { .. } => value.spring_coefficients().is_some(),
        _ => true,
    }
}

fn curve_value_compatible(left: &TemporalValue, right: &TemporalValue) -> bool {
    match (left, right) {
        (TemporalValue::Length { value: left }, TemporalValue::Length { value: right }) => {
            left.unit == right.unit
        }
        (TemporalValue::Point { value: left }, TemporalValue::Point { value: right }) => {
            left.x.unit == right.x.unit && left.y.unit == right.y.unit
        }
        _ => left.value_type() == right.value_type(),
    }
}
