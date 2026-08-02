use veac_ir::{RationalTime, TimeRange};

use crate::{ProviderError, ProviderErrorKind, ProviderResult};

pub(super) fn range_to_timebase(value: TimeRange, timebase: u32) -> ProviderResult<TimeRange> {
    TimeRange::new(
        convert(value.start, timebase)?,
        convert(value.duration, timebase)?,
    )
    .map_err(|error| {
        ProviderError::with_source(
            ProviderErrorKind::InvalidContract,
            "provider range is invalid in the project timebase",
            error,
        )
    })
}

pub(super) fn convert(value: RationalTime, timebase: u32) -> ProviderResult<RationalTime> {
    if value.timescale == 0 || timebase == 0 {
        return invalid("provider timebase must be positive");
    }
    let numerator = i128::from(value.value) * i128::from(timebase);
    let divisor = i128::from(value.timescale);
    if numerator % divisor != 0 {
        return invalid("provider time is not exactly representable in the project timebase");
    }
    let converted = i64::try_from(numerator / divisor)
        .map_err(|_| invalid_error("provider time overflows the project timebase"))?;
    RationalTime::new(converted, timebase).map_err(|error| {
        ProviderError::with_source(
            ProviderErrorKind::InvalidContract,
            "provider time is invalid in the project timebase",
            error,
        )
    })
}

pub(super) fn clip_time(
    value: RationalTime,
    binding: super::ClipTimeBinding,
    timebase: u32,
) -> ProviderResult<RationalTime> {
    let sample = convert(value, timebase)?;
    let provider = convert(binding.provider_origin, timebase)?;
    let local = convert(binding.clip_local_origin, timebase)?;
    let mapped = i128::from(sample.value) - i128::from(provider.value) + i128::from(local.value);
    let mapped = i64::try_from(mapped)
        .map_err(|_| invalid_error("provider clip-local time mapping overflowed"))?;
    RationalTime::new(mapped, timebase).map_err(|error| {
        ProviderError::with_source(
            ProviderErrorKind::InvalidContract,
            "provider clip-local time mapping is invalid",
            error,
        )
    })
}

fn invalid<T>(message: &str) -> ProviderResult<T> {
    Err(invalid_error(message))
}

fn invalid_error(message: &str) -> ProviderError {
    ProviderError::new(ProviderErrorKind::InvalidContract, message)
}

#[cfg(test)]
#[path = "time/tests.rs"]
mod tests;
