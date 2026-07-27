use veac_ir::{RationalTime, TimeRange, MAX_SAFE_INTEGER};

use crate::{
    OtioError, OtioRationalTime, OtioTimeRange, OTIO_RATIONAL_TIME_SCHEMA, OTIO_TIME_RANGE_SCHEMA,
};

pub(crate) fn export_time(value: RationalTime) -> Result<OtioRationalTime, OtioError> {
    if value.value.unsigned_abs() > MAX_SAFE_INTEGER || value.timescale == 0 {
        return Err(OtioError::time(
            "canonical time exceeds OTIO's exact number domain",
        ));
    }
    Ok(OtioRationalTime {
        schema: OTIO_RATIONAL_TIME_SCHEMA.to_owned(),
        value: value.value as f64,
        rate: f64::from(value.timescale),
    })
}

pub(crate) fn export_range(value: TimeRange) -> Result<OtioTimeRange, OtioError> {
    Ok(OtioTimeRange {
        schema: OTIO_TIME_RANGE_SCHEMA.to_owned(),
        start_time: export_time(value.start)?,
        duration: export_time(value.duration)?,
    })
}

pub(crate) fn import_time(
    value: OtioRationalTime,
    timebase: u32,
) -> Result<RationalTime, OtioError> {
    validate_time(&value)?;
    if timebase == 0 {
        return Err(OtioError::time("target timebase must be greater than zero"));
    }
    let (value_num, value_den) = decimal_ratio(value.value)?;
    let (rate_num, rate_den) = decimal_ratio(value.rate)?;
    let numerator = value_num
        .checked_mul(rate_den)
        .and_then(|item| item.checked_mul(i128::from(timebase)))
        .ok_or_else(|| OtioError::time("OTIO time arithmetic overflowed"))?;
    let denominator = value_den
        .checked_mul(rate_num)
        .ok_or_else(|| OtioError::time("OTIO time arithmetic overflowed"))?;
    if numerator % denominator != 0 {
        return Err(OtioError::time(
            "OTIO time cannot be represented exactly at the target timebase",
        ));
    }
    let ticks = i64::try_from(numerator / denominator)
        .map_err(|_| OtioError::time("OTIO time exceeds the canonical range"))?;
    RationalTime::new(ticks, timebase).map_err(OtioError::time)
}

pub(crate) fn import_range(value: OtioTimeRange, timebase: u32) -> Result<TimeRange, OtioError> {
    if value.schema != OTIO_TIME_RANGE_SCHEMA {
        return Err(OtioError::time("unsupported OTIO time-range schema"));
    }
    let start = import_time(value.start_time, timebase)?;
    let duration = import_time(value.duration, timebase)?;
    TimeRange::new(start, duration).map_err(OtioError::time)
}

pub(crate) fn validate_time(value: &OtioRationalTime) -> Result<(), OtioError> {
    if value.schema != OTIO_RATIONAL_TIME_SCHEMA
        || !value.value.is_finite()
        || !value.rate.is_finite()
        || value.rate <= 0.0
        || value.value.abs() > MAX_SAFE_INTEGER as f64
        || value.rate > MAX_SAFE_INTEGER as f64
    {
        return Err(OtioError::time(
            "rational time must use RationalTime.1 and finite exact-domain numbers",
        ));
    }
    Ok(())
}

pub(crate) fn validate_range(value: &OtioTimeRange) -> Result<(), OtioError> {
    if value.schema != OTIO_TIME_RANGE_SCHEMA || value.duration.value <= 0.0 {
        return Err(OtioError::time(
            "time range must use TimeRange.1 with positive duration",
        ));
    }
    validate_time(&value.start_time)?;
    validate_time(&value.duration)
}

fn decimal_ratio(value: f64) -> Result<(i128, i128), OtioError> {
    debug_assert!(value.is_finite());
    let text = value.to_string();
    let negative = text.starts_with('-');
    let unsigned = text.trim_start_matches(['-', '+']);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let digits = format!("{whole}{fraction}");
    let mut numerator = digits
        .parse::<i128>()
        .expect("finite f64 decimal digits fit in i128");
    if negative {
        numerator = -numerator;
    }
    Ok((numerator, power10(fraction.len() as u32)?))
}

fn power10(exponent: u32) -> Result<i128, OtioError> {
    10_i128
        .checked_pow(exponent)
        .ok_or_else(|| OtioError::time("OTIO time scale overflowed"))
}
