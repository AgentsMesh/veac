use crate::{Rational, RationalTime};

pub fn duration_within_seconds(duration: RationalTime, seconds: u64) -> bool {
    let Ok(value) = u128::try_from(duration.value) else {
        return false;
    };
    duration.is_valid()
        && value <= u128::from(seconds).saturating_mul(u128::from(duration.timescale))
}

pub fn units_for_duration(duration: RationalTime, rate: Rational) -> Option<u128> {
    if !duration.is_valid() || duration.value <= 0 || !rate.is_positive() {
        return None;
    }
    let value = u128::try_from(duration.value).ok()?;
    let rate_numerator = u128::try_from(rate.numerator).ok()?;
    let numerator = value.saturating_mul(rate_numerator);
    let denominator = u128::from(duration.timescale).checked_mul(u128::from(rate.denominator))?;
    Some(ceil_div(numerator, denominator))
}

pub fn samples_for_duration(duration: RationalTime, sample_rate: u32) -> Option<u128> {
    if sample_rate == 0 {
        return None;
    }
    units_for_duration(
        duration,
        Rational {
            numerator: i64::from(sample_rate),
            denominator: 1,
        },
    )
}

pub fn channel_samples_for_duration(
    duration: RationalTime,
    sample_rate: u32,
    channels: u8,
) -> Option<u128> {
    if channels == 0 {
        return None;
    }
    Some(samples_for_duration(duration, sample_rate)?.saturating_mul(u128::from(channels)))
}

pub fn pixel_frames(frames: u128, width: u32, height: u32) -> u128 {
    frames
        .saturating_mul(u128::from(width))
        .saturating_mul(u128::from(height))
}

pub fn scaled_duration_within_seconds(
    record: RationalTime,
    rate: Rational,
    repeat: u32,
    seconds: u64,
) -> bool {
    if !record.is_valid() || record.value <= 0 || !rate.is_positive() || repeat == 0 {
        return false;
    }
    let left = u128::try_from(record.value)
        .ok()
        .and_then(|value| value.checked_mul(rate.numerator as u128));
    let right = u128::from(seconds)
        .checked_mul(u128::from(record.timescale))
        .and_then(|value| value.checked_mul(u128::from(rate.denominator)))
        .and_then(|value| value.checked_mul(u128::from(repeat)));
    left.zip(right).is_some_and(|(left, right)| left <= right)
}

pub(crate) fn ceil_div(numerator: u128, denominator: u128) -> u128 {
    if denominator == 0 {
        return u128::MAX;
    }
    numerator / denominator + u128::from(numerator % denominator != 0)
}

#[cfg(test)]
#[path = "arithmetic_tests.rs"]
mod tests;
