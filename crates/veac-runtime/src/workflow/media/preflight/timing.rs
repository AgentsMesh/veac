use std::cmp::Ordering;

use veac_ir::{MediaProbeSnapshot, ProbedStream, RationalTime};

use super::{invalid, WorkflowResult};

pub(super) fn require_range(
    probe: &MediaProbeSnapshot,
    stream: &ProbedStream,
    requested_start: RationalTime,
    requested_end: RationalTime,
) -> WorkflowResult<()> {
    let stream_start = start(stream);
    if !matches!(
        requested_start.partial_cmp(&stream_start),
        Some(Ordering::Greater | Ordering::Equal)
    ) || !available_end_covers(probe, stream, requested_end, false)
    {
        return invalid("artifact source range is outside the selected stream duration");
    }
    Ok(())
}

pub(super) fn require_point(
    probe: &MediaProbeSnapshot,
    stream: &ProbedStream,
    point: RationalTime,
) -> WorkflowResult<()> {
    if !matches!(
        point.partial_cmp(&start(stream)),
        Some(Ordering::Greater | Ordering::Equal)
    ) || !available_end_covers(probe, stream, point, true)
    {
        return invalid("thumbnail time is outside the selected stream duration");
    }
    Ok(())
}

fn available_end_covers(
    probe: &MediaProbeSnapshot,
    stream: &ProbedStream,
    requested: RationalTime,
    strict: bool,
) -> bool {
    match stream.duration {
        Some(duration) => sum_covers(stream.start_time, duration, requested, strict),
        None if stream.start_time.is_none_or(|value| value.value == 0) => {
            probe.container_duration.is_some_and(|duration| {
                let comparison = duration.partial_cmp(&requested);
                comparison == Some(Ordering::Greater)
                    || !strict && comparison == Some(Ordering::Equal)
            })
        }
        None => false,
    }
}

fn sum_covers(
    start: Option<RationalTime>,
    duration: RationalTime,
    requested: RationalTime,
    strict: bool,
) -> bool {
    let start = start.unwrap_or_else(zero);
    let left = nonnegative(requested)
        .and_then(|value| value.checked_mul(u128::from(start.timescale)))
        .and_then(|value| value.checked_mul(u128::from(duration.timescale)));
    let start_term = nonnegative(start)
        .and_then(|value| value.checked_mul(u128::from(requested.timescale)))
        .and_then(|value| value.checked_mul(u128::from(duration.timescale)));
    let duration_term = nonnegative(duration)
        .and_then(|value| value.checked_mul(u128::from(requested.timescale)))
        .and_then(|value| value.checked_mul(u128::from(start.timescale)));
    left.zip(start_term.zip(duration_term))
        .and_then(|(left, terms)| terms.0.checked_add(terms.1).map(|right| (left, right)))
        .is_some_and(|(left, right)| if strict { left < right } else { left <= right })
}

fn start(stream: &ProbedStream) -> RationalTime {
    stream.start_time.unwrap_or_else(zero)
}

fn zero() -> RationalTime {
    RationalTime {
        value: 0,
        timescale: 1,
    }
}

fn nonnegative(value: RationalTime) -> Option<u128> {
    value
        .is_valid()
        .then(|| u128::try_from(value.value).ok())
        .flatten()
}

#[cfg(test)]
#[path = "timing/tests.rs"]
mod tests;
