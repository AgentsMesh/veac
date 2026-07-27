use veac_ir::{Rational, RationalTime};

use super::ProbeError;

pub(super) fn optional_time(
    field: &'static str,
    raw: Option<&str>,
    positive: bool,
) -> Result<Option<RationalTime>, ProbeError> {
    let raw = match raw {
        Some(value) if !value.is_empty() && value != "N/A" => value,
        _ => return Ok(None),
    };
    let time = decimal_time(field, raw)?;
    if (positive && time.value <= 0) || (!positive && time.value < 0) {
        return Err(invalid(field, raw));
    }
    Ok(Some(time))
}

pub(super) fn sample_aspect_ratio(raw: Option<&str>) -> Result<Rational, ProbeError> {
    let raw = match raw {
        Some(value) if !value.is_empty() && value != "N/A" => value,
        _ => {
            return Ok(Rational {
                numerator: 1,
                denominator: 1,
            })
        }
    };
    let (numerator, denominator) = match raw.split_once(':') {
        Some(parts) => parts,
        None => return Err(invalid("sample_aspect_ratio", raw)),
    };
    let numerator = match numerator.parse::<i64>() {
        Ok(value) => value,
        Err(_) => return Err(invalid("sample_aspect_ratio", raw)),
    };
    let denominator = match denominator.parse::<u32>() {
        Ok(value) => value,
        Err(_) => return Err(invalid("sample_aspect_ratio", raw)),
    };
    let ratio = match Rational::new(numerator, denominator) {
        Ok(value) => value,
        Err(_) => return Err(invalid("sample_aspect_ratio", raw)),
    };
    if ratio.is_positive() {
        Ok(ratio)
    } else {
        Err(invalid("sample_aspect_ratio", raw))
    }
}

pub(super) fn positive_ratio(
    field: &'static str,
    raw: Option<&str>,
) -> Result<Rational, ProbeError> {
    let raw = raw
        .filter(|value| !value.is_empty() && *value != "N/A")
        .unwrap_or("");
    let Some((numerator, denominator)) = raw.split_once('/') else {
        return Err(invalid(field, raw));
    };
    let ratio = numerator
        .parse::<i64>()
        .ok()
        .zip(denominator.parse::<u32>().ok())
        .and_then(|(numerator, denominator)| Rational::new(numerator, denominator).ok());
    match ratio {
        Some(value) if value.is_positive() => Ok(value),
        _ => Err(invalid(field, raw)),
    }
}

pub(super) fn optional_positive_ratio(
    field: &'static str,
    raw: Option<&str>,
) -> Result<Option<Rational>, ProbeError> {
    let Some(raw) = raw.filter(|value| !value.is_empty() && *value != "N/A") else {
        return Ok(None);
    };
    if matches!(raw, "0/0" | "0/1") {
        return Ok(None);
    }
    positive_ratio(field, Some(raw)).map(Some)
}

fn decimal_time(field: &'static str, raw: &str) -> Result<RationalTime, ProbeError> {
    let (negative, unsigned) = match raw.strip_prefix('-') {
        Some(value) => (true, value),
        None => (false, raw),
    };
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty() && fraction.is_empty() || !digits(whole) || !digits(fraction) {
        return Err(invalid(field, raw));
    }
    let fraction = fraction.trim_end_matches('0');
    let timescale = match 10_u32.checked_pow(fraction.len() as u32) {
        Some(value) => value,
        None => return Err(invalid(field, raw)),
    };
    let whole = match whole.parse::<i128>() {
        Ok(value) => value,
        Err(_) => return Err(invalid(field, raw)),
    };
    let fractional = if fraction.is_empty() {
        0
    } else {
        match fraction.parse::<i128>() {
            Ok(value) => value,
            Err(_) => return Err(invalid(field, raw)),
        }
    };
    let scaled = match whole.checked_mul(i128::from(timescale)) {
        Some(value) => value,
        None => return Err(invalid(field, raw)),
    };
    let unsigned_value = match scaled.checked_add(fractional) {
        Some(value) => value,
        None => return Err(invalid(field, raw)),
    };
    let signed = if negative {
        -unsigned_value
    } else {
        unsigned_value
    };
    let value = match i64::try_from(signed) {
        Ok(value) => value,
        Err(_) => return Err(invalid(field, raw)),
    };
    let divisor = gcd(value.unsigned_abs(), u64::from(timescale));
    let divisor_u32 = divisor as u32;
    let divisor_i64 = i64::from(divisor_u32);
    match RationalTime::new(value / divisor_i64, timescale / divisor_u32) {
        Ok(value) => Ok(value),
        Err(_) => Err(invalid(field, raw)),
    }
}

fn digits(value: &str) -> bool {
    for byte in value.bytes() {
        if !byte.is_ascii_digit() {
            return false;
        }
    }
    true
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}

fn invalid(field: &'static str, value: &str) -> ProbeError {
    ProbeError::InvalidField {
        field,
        value: value.to_owned(),
    }
}

#[cfg(test)]
#[path = "time/tests.rs"]
mod tests;
