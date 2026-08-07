use super::super::super::super::super::{ResidualRuntimeValue, ResidualizationError};
use super::error;
use crate::program::expression::Value;
use veac_ir::{Interpolation, TemporalCurveKey, TemporalCurvePosition, TemporalValue};

pub(super) fn curve_keys(
    value: &ResidualRuntimeValue,
    interpolation: Interpolation,
    span: std::ops::Range<usize>,
) -> Result<Vec<TemporalCurveKey>, ResidualizationError> {
    match value {
        ResidualRuntimeValue::Concrete(Value::List(values)) => values
            .values()
            .iter()
            .map(|value| concrete_key(value, interpolation.clone(), span.clone()))
            .collect(),
        ResidualRuntimeValue::Sequence { values, .. } => values
            .iter()
            .map(|value| runtime_key(value, interpolation.clone(), span.clone()))
            .collect(),
        _ => Err(key_error(span)),
    }
}

fn concrete_key(
    value: &Value,
    interpolation: Interpolation,
    span: std::ops::Range<usize>,
) -> Result<TemporalCurveKey, ResidualizationError> {
    let Value::Tuple(value) = value else {
        return Err(key_error(span));
    };
    make_key(
        concrete_temporal(&value.values()[0], span.clone())?,
        concrete_temporal(&value.values()[1], span.clone())?,
        interpolation,
        span,
    )
}

fn runtime_key(
    value: &ResidualRuntimeValue,
    interpolation: Interpolation,
    span: std::ops::Range<usize>,
) -> Result<TemporalCurveKey, ResidualizationError> {
    if let ResidualRuntimeValue::Concrete(value) = value {
        return concrete_key(value, interpolation, span);
    }
    let ResidualRuntimeValue::Sequence { values, .. } = value else {
        return Err(key_error(span));
    };
    if values.len() != 2 {
        return Err(key_error(span));
    }
    make_key(
        runtime_temporal(&values[0], span.clone())?,
        runtime_temporal(&values[1], span.clone())?,
        interpolation,
        span,
    )
}

fn make_key(
    position: TemporalValue,
    value: TemporalValue,
    interpolation: Interpolation,
    span: std::ops::Range<usize>,
) -> Result<TemporalCurveKey, ResidualizationError> {
    let position = match position {
        TemporalValue::Scalar { value } => TemporalCurvePosition::Scalar { value },
        TemporalValue::Time { value } => TemporalCurvePosition::Time { value },
        _ => {
            return Err(error(
                "RESIDUAL_CURVE_POSITION",
                "invalid curve position",
                span,
            ))
        }
    };
    Ok(TemporalCurveKey {
        position,
        value,
        interpolation,
    })
}

pub(super) fn scalar(
    value: &ResidualRuntimeValue,
    span: std::ops::Range<usize>,
) -> Result<f64, ResidualizationError> {
    match runtime_temporal(value, span.clone())? {
        TemporalValue::Scalar { value } => Ok(value),
        _ => Err(error(
            "RESIDUAL_CURVE_INTERPOLATION",
            "curve interpolation parameters must be Build-stage scalar values",
            span,
        )),
    }
}

fn runtime_temporal(
    value: &ResidualRuntimeValue,
    span: std::ops::Range<usize>,
) -> Result<TemporalValue, ResidualizationError> {
    match value {
        ResidualRuntimeValue::Concrete(value) => concrete_temporal(value, span),
        ResidualRuntimeValue::TemporalConstant(value) => Ok(value.clone()),
        _ => Err(error(
            "RESIDUAL_CURVE_DYNAMIC_KEY",
            "curve keys must be known at Build stage",
            span,
        )),
    }
}

fn concrete_temporal(
    value: &Value,
    span: std::ops::Range<usize>,
) -> Result<TemporalValue, ResidualizationError> {
    super::super::super::super::super::convert::temporal_value(value, span).map(|value| value.1)
}

pub(super) fn key_error(span: std::ops::Range<usize>) -> ResidualizationError {
    error(
        "RESIDUAL_CURVE_KEYS",
        "curve keys must be a non-empty Build-stage list of (position, value) tuples",
        span,
    )
}
