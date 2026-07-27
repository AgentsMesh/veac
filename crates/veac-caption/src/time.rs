use veac_ir::{RationalTime, TimeRange};

use crate::CaptionError;

pub(crate) fn range_from_millis(
    start: u64,
    end: u64,
    timescale: u32,
) -> Result<TimeRange, CaptionError> {
    if end <= start {
        return Err(CaptionError::time("cue end must be after cue start"));
    }
    let start = from_millis(start, timescale)?;
    let end = from_millis(end, timescale)?;
    let duration = RationalTime::new(end.value - start.value, timescale)
        .map_err(|error| CaptionError::time(error.to_string()))?;
    TimeRange::new(start, duration).map_err(|error| CaptionError::time(error.to_string()))
}

pub(crate) fn from_millis(value: u64, timescale: u32) -> Result<RationalTime, CaptionError> {
    if timescale == 0 {
        return Err(CaptionError::time("timescale must be greater than zero"));
    }
    let scaled = i128::from(value) * i128::from(timescale);
    if scaled % 1000 != 0 {
        return Err(CaptionError::time(format!(
            "{value}ms is not exactly representable at timescale {timescale}"
        )));
    }
    let exact = i64::try_from(scaled / 1000)
        .map_err(|_| CaptionError::time("timestamp is outside the supported integer range"))?;
    RationalTime::new(exact, timescale).map_err(|error| CaptionError::time(error.to_string()))
}

pub(crate) fn to_millis(value: RationalTime) -> Result<u64, CaptionError> {
    if !value.is_valid() || value.value < 0 {
        return Err(CaptionError::time(
            "timestamp must be valid and nonnegative",
        ));
    }
    let scaled = i128::from(value.value) * 1000;
    if scaled % i128::from(value.timescale) != 0 {
        return Err(CaptionError::time(format!(
            "{}/{}s is not exactly representable in milliseconds",
            value.value, value.timescale
        )));
    }
    u64::try_from(scaled / i128::from(value.timescale))
        .map_err(|_| CaptionError::time("timestamp is outside the supported millisecond range"))
}

pub(crate) fn range_to_millis(range: TimeRange) -> Result<(u64, u64), CaptionError> {
    Ok((
        to_millis(range.start)?,
        to_millis(
            range
                .end()
                .map_err(|error| CaptionError::time(error.to_string()))?,
        )?,
    ))
}
