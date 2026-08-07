use std::ops::Range;

use super::super::{ExactNumber, PrimitiveType, Value, ValueType};
use super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::DomainType;
use veac_ir::{Color, Length, LengthUnit, RationalTime, TemporalType, TemporalValue};

pub(super) fn value_type(value: &ValueType) -> Option<TemporalType> {
    if let Some(value) = value.as_primitive() {
        return Some(match value {
            PrimitiveType::Boolean => TemporalType::Boolean,
            PrimitiveType::Integer => TemporalType::Integer,
            PrimitiveType::Scalar | PrimitiveType::Percent => TemporalType::Scalar,
            PrimitiveType::Time => TemporalType::Time,
            PrimitiveType::Length => TemporalType::Length,
            PrimitiveType::Angle => TemporalType::Angle,
            PrimitiveType::Color => TemporalType::Color,
            PrimitiveType::Text => TemporalType::Text,
            PrimitiveType::Identifier => return None,
        });
    }
    match value.as_domain()? {
        DomainType::Vector => Some(TemporalType::Vec2),
        DomainType::Point => Some(TemporalType::Point),
        DomainType::Rect => Some(TemporalType::Rect),
        _ => None,
    }
}

pub(super) fn runtime_type(
    value: &ResidualRuntimeValue,
    span: Range<usize>,
) -> Result<TemporalType, ResidualizationError> {
    match value {
        ResidualRuntimeValue::Residual(value) => Ok(value.value_type),
        ResidualRuntimeValue::TemporalConstant(value) => Ok(value.value_type()),
        ResidualRuntimeValue::Concrete(value) => temporal_value(value, span).map(|value| value.0),
        ResidualRuntimeValue::Sequence { .. } => Err(ResidualizationError::new(
            "RESIDUAL_VALUE_UNSUPPORTED",
            "structural values cannot become Temporal nodes",
            span,
        )),
    }
}

pub(super) fn temporal_value(
    value: &Value,
    span: Range<usize>,
) -> Result<(TemporalType, TemporalValue), ResidualizationError> {
    let converted = match value {
        Value::Bool(value) => TemporalValue::Boolean { value: *value },
        Value::Integer(value) => TemporalValue::Integer { value: *value },
        Value::Scalar(value) => TemporalValue::Scalar {
            value: number(*value, 1.0, span.clone())?,
        },
        Value::Percent(value) => TemporalValue::Scalar {
            value: number(*value, 0.01, span.clone())?,
        },
        Value::Time(value) => TemporalValue::Time {
            value: time(*value, span.clone())?,
        },
        Value::Length(value) => TemporalValue::Length {
            value: Length {
                value: number(*value, 1.0, span.clone())?,
                unit: LengthUnit::Pixels,
            },
        },
        Value::Angle(value) => TemporalValue::Angle {
            degrees: number(*value, 1.0, span.clone())?,
        },
        Value::Text(value) => TemporalValue::Text {
            value: value.to_string(),
        },
        Value::Color(value) => TemporalValue::Color {
            value: color(value, span.clone())?,
        },
        _ => {
            return Err(ResidualizationError::new(
                "RESIDUAL_VALUE_UNSUPPORTED",
                format!("{} cannot be materialized in Temporal IR", value.kind()),
                span,
            ))
        }
    };
    Ok((converted.value_type(), converted))
}

fn number(value: ExactNumber, scale: f64, span: Range<usize>) -> Result<f64, ResidualizationError> {
    let value = value.numerator() as f64 / value.denominator() as f64 * scale;
    if !value.is_finite() {
        return Err(ResidualizationError::new(
            "RESIDUAL_NUMBER_RANGE",
            "exact number is outside Temporal IR's finite numeric range",
            span,
        ));
    }
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn time(value: ExactNumber, span: Range<usize>) -> Result<RationalTime, ResidualizationError> {
    let numerator = i64::try_from(value.numerator()).ok();
    let denominator = u32::try_from(value.denominator()).ok();
    numerator
        .zip(denominator)
        .and_then(|(value, scale)| RationalTime::new(value, scale).ok())
        .ok_or_else(|| {
            ResidualizationError::new(
                "RESIDUAL_TIME_RANGE",
                "exact time is outside Temporal IR's rational range",
                span,
            )
        })
}

fn color(value: &str, span: Range<usize>) -> Result<Color, ResidualizationError> {
    let raw = value.strip_prefix('#').unwrap_or("");
    let bytes = match raw.len() {
        6 => [hex(&raw[0..2]), hex(&raw[2..4]), hex(&raw[4..6]), Some(255)],
        8 => [
            hex(&raw[0..2]),
            hex(&raw[2..4]),
            hex(&raw[4..6]),
            hex(&raw[6..8]),
        ],
        _ => [None; 4],
    };
    let [Some(red), Some(green), Some(blue), Some(alpha)] = bytes else {
        return Err(ResidualizationError::new(
            "RESIDUAL_COLOR_LITERAL",
            "Temporal IR colors require #rrggbb or #rrggbbaa",
            span,
        ));
    };
    Ok(Color {
        red,
        green,
        blue,
        alpha,
    })
}

fn hex(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}
