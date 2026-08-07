use crate::{RationalTime, TemporalEvaluationError};

pub(super) fn number(value: f64, pointer: &str) -> Result<f64, TemporalEvaluationError> {
    if !value.is_finite() {
        Err(error(
            pointer,
            "numeric operation produced a non-finite value",
        ))
    } else if value == 0.0 {
        Ok(0.0)
    } else {
        Ok(value)
    }
}

pub(super) fn integer(value: i128, pointer: &str) -> Result<i64, TemporalEvaluationError> {
    let value = match i64::try_from(value) {
        Ok(value) => value,
        Err(_) => return Err(error(pointer, "integer operation overflowed")),
    };
    if value.unsigned_abs() > crate::MAX_SAFE_INTEGER {
        Err(error(
            pointer,
            "integer operation left the exact numeric range",
        ))
    } else {
        Ok(value)
    }
}

pub(super) fn time(
    value: i128,
    timescale: u32,
    pointer: &str,
) -> Result<RationalTime, TemporalEvaluationError> {
    match RationalTime::new(integer(value, pointer)?, timescale) {
        Ok(value) => Ok(value),
        Err(cause) => Err(error(pointer, &cause.to_string())),
    }
}

pub(super) fn error(pointer: &str, message: &str) -> TemporalEvaluationError {
    TemporalEvaluationError::new("TEMPORAL_EVALUATION_VALUE", pointer, message)
}

pub(super) fn contract(pointer: &str) -> TemporalEvaluationError {
    TemporalEvaluationError::new(
        "TEMPORAL_EVALUATION_CONTRACT",
        pointer,
        "verified operation received incompatible values",
    )
}
