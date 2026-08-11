use veac_artifact::SourceClockSpec;
use veac_build::ProjectBackendError;
use veac_ir::{Rational, RationalTime, TimeRange};
use veac_project::{ProjectRational, ProjectSourceClock};

use super::super::super::{failed, ProjectOptionExt, ProjectResultExt};

pub(super) fn clock(value: ProjectSourceClock) -> Result<SourceClockSpec, ProjectBackendError> {
    match value {
        ProjectSourceClock::Identity { duration } => Ok(SourceClockSpec::Identity {
            duration: time(duration)?,
        }),
        ProjectSourceClock::Bounded { start, duration } => {
            let (start, duration) = common_times(start, duration)?;
            let logical_range =
                TimeRange::new(start, duration).project_context("invalid derivation clock")?;
            Ok(SourceClockSpec::Bounded { logical_range })
        }
    }
}

pub(super) fn rate(value: ProjectRational) -> Result<Rational, ProjectBackendError> {
    let denominator = u32::try_from(value.denominator)
        .project_context("derivation rate denominator exceeds u32")?;
    let converted =
        Rational::new(value.numerator, denominator).project_context("invalid derivation rate")?;
    if converted.numerator != value.numerator
        || u64::from(converted.denominator) != value.denominator
    {
        return Err(failed("derivation rate must be canonical"));
    }
    Ok(converted)
}

pub(super) fn time(value: ProjectRational) -> Result<RationalTime, ProjectBackendError> {
    let timescale = u32::try_from(value.denominator)
        .project_context("derivation time denominator exceeds u32")?;
    RationalTime::new(value.numerator, timescale).project_context("invalid derivation time")
}

pub(super) fn common_times(
    start: ProjectRational,
    duration: ProjectRational,
) -> Result<(RationalTime, RationalTime), ProjectBackendError> {
    let divisor = gcd(start.denominator, duration.denominator);
    let timescale = start
        .denominator
        .checked_div(divisor)
        .and_then(|value| value.checked_mul(duration.denominator))
        .and_then(|value| u32::try_from(value).ok())
        .project_required("derivation clock common timescale exceeds u32")?;
    let convert = |value: ProjectRational| {
        let multiplier = u64::from(timescale) / value.denominator;
        let scaled = i128::from(value.numerator) * i128::from(multiplier);
        let scaled =
            i64::try_from(scaled).project_context("derivation clock numerator overflow")?;
        RationalTime::new(scaled, timescale).project_context("invalid derivation clock")
    };
    Ok((convert(start)?, convert(duration)?))
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}
