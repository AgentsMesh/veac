use std::cmp::Ordering;

use veac_ir::{Clip, RationalTime, TimeRange};

use crate::{ResolutionDiagnostic, ResolutionErrorKind};

use super::{BoundContext, PlanResolver};

impl PlanResolver<'_> {
    pub(super) fn check_point_duration(
        &mut self,
        source_time: RationalTime,
        duration: Option<RationalTime>,
        context: BoundContext<'_>,
        media_type: &str,
    ) {
        let Some(duration) = duration else {
            self.missing_duration(context.clip, context.path, media_type);
            return;
        };
        if source_time.value < 0 {
            if !context.outside.allows_before() {
                self.out_of_bounds(
                    context.clip,
                    context.path,
                    &format!("{media_type} source time"),
                );
            }
            return;
        }
        match source_time.partial_cmp(&duration) {
            None => self.missing_duration(context.clip, context.path, media_type),
            Some(Ordering::Less) => {}
            Some(_) if context.outside.allows_after() => {}
            Some(_) => self.out_of_bounds(
                context.clip,
                context.path,
                &format!("{media_type} source time"),
            ),
        }
    }

    pub(super) fn check_range_duration(
        &mut self,
        range: TimeRange,
        duration: Option<RationalTime>,
        context: BoundContext<'_>,
        media_type: &str,
        upper_inclusive: bool,
    ) {
        if range.start.value < 0 && !context.outside.allows_before() {
            self.out_of_bounds(context.clip, context.path, media_type);
            return;
        }
        let Some(duration) = duration else {
            self.missing_duration(context.clip, context.path, media_type);
            return;
        };
        match range.end().ok().and_then(|end| end.partial_cmp(&duration)) {
            Some(Ordering::Less) => {}
            Some(Ordering::Equal) if !upper_inclusive || context.outside.allows_after() => {}
            Some(Ordering::Greater) if context.outside.allows_after() => {}
            Some(Ordering::Equal | Ordering::Greater) => {
                self.out_of_bounds(context.clip, context.path, media_type)
            }
            None => self.push(ResolutionDiagnostic::new(
                ResolutionErrorKind::TimeArithmetic,
                "SOURCE_BOUND_TIME_ARITHMETIC",
                Some(context.clip.id.to_string()),
                format!("{}/source_mapping", context.path),
                "source range and probe duration cannot be compared exactly",
            )),
        }
    }

    fn missing_duration(&mut self, clip: &Clip, path: &str, media_type: &str) {
        self.push(ResolutionDiagnostic::new(
            ResolutionErrorKind::SourceDurationUnavailable,
            "SOURCE_DURATION_UNAVAILABLE",
            Some(clip.id.to_string()),
            format!("{path}/source"),
            format!("{media_type} probe has no duration required for source bounds"),
        ));
    }

    fn out_of_bounds(&mut self, clip: &Clip, path: &str, subject: &str) {
        self.push(ResolutionDiagnostic::new(
            ResolutionErrorKind::SourceRangeOutOfBounds,
            "SOURCE_RANGE_OUT_OF_BOUNDS",
            Some(clip.id.to_string()),
            format!("{path}/source_mapping"),
            format!("{subject} exceeds the observed material duration"),
        ));
    }
}
