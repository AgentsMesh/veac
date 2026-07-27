use std::cmp::Ordering;

use veac_ir::{RationalTime, Rect, TimeRange};

use crate::{ProviderError, ProviderErrorKind, ProviderResult};

pub(crate) fn invalid<T>(message: impl Into<String>) -> ProviderResult<T> {
    Err(ProviderError::new(
        ProviderErrorKind::InvalidContract,
        message,
    ))
}

pub(crate) fn text(value: &str, label: &str) -> ProviderResult<()> {
    if value.trim().is_empty() {
        invalid(format!("{label} must be non-empty"))
    } else {
        Ok(())
    }
}

pub(crate) fn language(value: &str) -> ProviderResult<()> {
    if value.len() > 63
        || value.split('-').any(|part| part.is_empty())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        invalid("language tags must be non-empty BCP-47-style identifiers")
    } else {
        Ok(())
    }
}

pub(crate) fn finite(value: f64, label: &str) -> ProviderResult<()> {
    if value.is_finite() {
        Ok(())
    } else {
        invalid(format!("{label} must be finite"))
    }
}

pub(crate) fn probability(value: f64, label: &str) -> ProviderResult<()> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        invalid(format!("{label} must be between zero and one"))
    }
}

pub(crate) fn positive(value: f64, label: &str) -> ProviderResult<()> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        invalid(format!("{label} must be positive"))
    }
}

pub(crate) fn time(value: RationalTime) -> ProviderResult<()> {
    if value.is_valid() {
        Ok(())
    } else {
        invalid("time must be an exact, valid rational value")
    }
}

pub(crate) fn range(value: TimeRange) -> ProviderResult<()> {
    if value.start.is_valid()
        && value.duration.is_valid()
        && value.start.timescale == value.duration.timescale
        && value.duration.value > 0
        && value.end().is_ok()
    {
        Ok(())
    } else {
        invalid("time range must have a valid start and positive duration")
    }
}

pub(crate) fn increasing(values: &[RationalTime]) -> ProviderResult<()> {
    for value in values {
        time(*value)?;
    }
    if values
        .windows(2)
        .all(|pair| pair[0].partial_cmp(&pair[1]) == Some(Ordering::Less))
    {
        Ok(())
    } else {
        invalid("sample times must be strictly increasing")
    }
}

pub(crate) fn non_overlapping(values: &[TimeRange]) -> ProviderResult<()> {
    for value in values {
        range(*value)?;
    }
    if values.windows(2).all(|pair| {
        pair[0]
            .end()
            .ok()
            .zip(Some(pair[1].start))
            .is_some_and(|(end, start)| end <= start)
    }) {
        Ok(())
    } else {
        invalid("time ranges must be ordered and non-overlapping")
    }
}

pub(crate) fn normalized_rect(value: Rect) -> ProviderResult<()> {
    let values = [value.x, value.y, value.width, value.height];
    if values.iter().all(|item| item.is_finite())
        && value.x >= 0.0
        && value.y >= 0.0
        && value.width > 0.0
        && value.height > 0.0
        && value.x + value.width <= 1.0
        && value.y + value.height <= 1.0
    {
        Ok(())
    } else {
        invalid("rectangle must be positive and contained in normalized coordinates")
    }
}
