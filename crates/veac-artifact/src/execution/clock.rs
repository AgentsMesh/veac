use std::cmp::Ordering;

use veac_ir::{RationalTime, TimeRange};

mod math;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceClock {
    timescale: u32,
    logical_range: Option<TimeRange>,
    physical_start: RationalTime,
}

impl SourceClock {
    pub fn identity(timescale: u32) -> ArtifactResult<Self> {
        let physical_start = RationalTime::zero(timescale).map_err(invalid_time)?;
        Ok(Self {
            timescale,
            logical_range: None,
            physical_start,
        })
    }

    pub fn bounded(logical_range: TimeRange, physical_start: RationalTime) -> ArtifactResult<Self> {
        let timescale = logical_range.start.timescale;
        if logical_range.start.value < 0
            || logical_range.duration.value <= 0
            || logical_range.duration.timescale != timescale
            || physical_start.value < 0
            || physical_start.timescale != timescale
            || logical_range.end().is_err()
        {
            return invalid("source clock must contain valid nonnegative aligned exact times");
        }
        Ok(Self {
            timescale,
            logical_range: Some(logical_range),
            physical_start,
        })
    }

    pub fn original_stream(
        duration: RationalTime,
        physical_start: Option<RationalTime>,
    ) -> ArtifactResult<Self> {
        let start =
            physical_start.unwrap_or(RationalTime::zero(duration.timescale).map_err(invalid_time)?);
        let (logical_range, physical_start) =
            math::stream_domain(duration, start).ok_or_else(|| {
                ArtifactError::new(
                    ArtifactErrorKind::InvalidContract,
                    "source stream clock is not representable",
                )
            })?;
        Self::bounded(logical_range, physical_start)
    }

    pub fn timescale(self) -> u32 {
        self.timescale
    }

    pub fn logical_range(self) -> Option<TimeRange> {
        self.logical_range
    }

    pub fn physical_start(self) -> RationalTime {
        self.physical_start
    }

    pub fn extended(self, before: RationalTime, after: RationalTime) -> ArtifactResult<Self> {
        let Some(range) = self.logical_range else {
            return invalid("source clock extension requires a bounded clock");
        };
        let extended = math::extended_range(range, before, after).ok_or_else(|| {
            ArtifactError::new(
                ArtifactErrorKind::InvalidContract,
                "source clock extension is not representable",
            )
        })?;
        let timescale = extended.start.timescale;
        Ok(Self {
            timescale,
            logical_range: Some(extended),
            physical_start: RationalTime::zero(timescale).map_err(invalid_time)?,
        })
    }

    pub fn map_point(self, logical: RationalTime) -> Option<RationalTime> {
        self.map(logical, false)
    }

    pub fn map_boundary(self, logical: RationalTime) -> Option<RationalTime> {
        self.map(logical, true)
    }

    pub fn covers_range(self, range: TimeRange) -> bool {
        let Ok(end) = range.end() else {
            return false;
        };
        self.map_boundary(range.start).is_some() && self.map_boundary(end).is_some()
    }

    fn map(self, logical: RationalTime, inclusive_end: bool) -> Option<RationalTime> {
        if !logical.is_valid() || self.logical_range.is_none() && logical.value < 0 {
            return None;
        }
        if self.logical_range.is_none() {
            return Some(logical);
        }
        let origin = match self.logical_range {
            Some(range) => {
                let end = range.end().ok()?;
                let upper = logical.partial_cmp(&end)?;
                if logical < range.start
                    || upper == Ordering::Greater
                    || (!inclusive_end && upper == Ordering::Equal)
                {
                    return None;
                }
                range.start
            }
            None => unreachable!("unbounded identity clocks return above"),
        };
        math::mapped_time(logical, origin, self.physical_start)
    }
}

fn invalid_time(error: veac_ir::TimeError) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::InvalidContract,
        "source clock timescale is invalid",
        error,
    )
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}
