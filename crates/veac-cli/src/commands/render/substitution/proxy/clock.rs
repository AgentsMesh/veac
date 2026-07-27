use veac_artifact::SourceClockSpec;
use veac_ir::{RationalTime, TimeRange};
use veac_plan::PlanInputId;

use crate::error::{CliError, CliResult};

pub(super) fn resolve(
    input_id: &PlanInputId,
    project_timebase: u32,
    container_duration: Option<RationalTime>,
    stream_start: Option<RationalTime>,
    stream_duration: Option<RationalTime>,
    role: &str,
) -> CliResult<SourceClockSpec> {
    let start = match stream_start {
        Some(value) if value.is_valid() && value.value >= 0 => Some(value),
        None => None,
        _ => return Err(error(input_id, role)),
    };
    let duration = stream_duration
        .or_else(|| {
            start
                .is_none_or(|value| value.value == 0)
                .then_some(container_duration)
                .flatten()
        })
        .filter(|value| value.is_valid() && value.value > 0)
        .and_then(|value| rescale(value, project_timebase))
        .ok_or_else(|| error(input_id, role))?;
    let Some(start) = start.filter(|value| value.value > 0) else {
        return Ok(SourceClockSpec::Identity { duration });
    };
    let start = rescale(start, project_timebase).ok_or_else(|| error(input_id, role))?;
    let logical_range = TimeRange::new(start, duration).map_err(|_| error(input_id, role))?;
    Ok(SourceClockSpec::Bounded { logical_range })
}

fn rescale(value: RationalTime, timescale: u32) -> Option<RationalTime> {
    if timescale == 0 || !value.is_valid() {
        return None;
    }
    let numerator = i128::from(value.value).checked_mul(i128::from(timescale))?;
    let denominator = i128::from(value.timescale);
    if numerator % denominator != 0 {
        return None;
    }
    RationalTime::new(i64::try_from(numerator / denominator).ok()?, timescale).ok()
}

fn error(input_id: &PlanInputId, role: &str) -> CliError {
    CliError::new(
        "PROXY_DURATION_UNAVAILABLE",
        format!("input {input_id} has no exact {role} source range"),
    )
}

#[cfg(test)]
#[path = "clock/tests.rs"]
mod tests;
