use veac_ir::{Clip, SourceMapping, SourceTimeMap};

use crate::{
    ResolutionDiagnostic, ResolutionErrorKind, ResolvedInput, ResolvedSourceMapping,
    ResolvedSourceTimeMap,
};

use super::{bounds::BoundContext, material::InputUsage, time, PlanResolver};

impl PlanResolver<'_> {
    pub(super) fn resolve_source_mapping(
        &mut self,
        mapping: &SourceMapping,
        clip: &Clip,
        input: &ResolvedInput,
        usage: InputUsage,
        path: &str,
    ) -> Option<ResolvedSourceMapping> {
        let Some(resolved) = resolve_mapping(mapping, clip) else {
            self.mapping_arithmetic_error(clip, path);
            return None;
        };
        let context = BoundContext::new(clip, path, resolved.out_of_range);
        match &resolved.time_map {
            ResolvedSourceTimeMap::Linear {
                source_range_per_repeat,
                ..
            } => self.check_source_bounds(input, *source_range_per_repeat, usage, context, false),
            ResolvedSourceTimeMap::Curve { segments } => {
                let bounds = curve_bounds(segments)?;
                if bounds.start == bounds.end {
                    self.check_source_point_bounds(input, bounds.start, usage, context)
                } else {
                    self.check_source_bounds(
                        input,
                        time::range_between(bounds.start, bounds.end)?,
                        usage,
                        context,
                        bounds.end_is_point,
                    );
                }
            }
        }
        Some(resolved)
    }

    pub(super) fn resolve_sequence_mapping(
        &mut self,
        mapping: &SourceMapping,
        clip: &Clip,
        duration: veac_ir::RationalTime,
        path: &str,
    ) -> Option<ResolvedSourceMapping> {
        let Some(resolved) = resolve_mapping(mapping, clip) else {
            self.mapping_arithmetic_error(clip, path);
            return None;
        };
        let context = BoundContext::new(clip, path, resolved.out_of_range);
        match &resolved.time_map {
            ResolvedSourceTimeMap::Linear {
                source_range_per_repeat,
                ..
            } => self.check_sequence_source_bounds(
                duration,
                *source_range_per_repeat,
                context,
                false,
            ),
            ResolvedSourceTimeMap::Curve { segments } => {
                let bounds = curve_bounds(segments)?;
                if bounds.start == bounds.end {
                    self.check_sequence_source_point_bounds(duration, bounds.start, context)
                } else {
                    self.check_sequence_source_bounds(
                        duration,
                        time::range_between(bounds.start, bounds.end)?,
                        context,
                        bounds.end_is_point,
                    );
                }
            }
        }
        Some(resolved)
    }

    fn mapping_arithmetic_error(&mut self, clip: &Clip, path: &str) {
        self.push(ResolutionDiagnostic::new(
            ResolutionErrorKind::TimeArithmetic,
            "SOURCE_TIME_ARITHMETIC",
            Some(clip.id.to_string()),
            format!("{path}/source_mapping"),
            "source-time mapping exceeds safe exact arithmetic bounds",
        ));
    }
}

fn resolve_mapping(mapping: &SourceMapping, clip: &Clip) -> Option<ResolvedSourceMapping> {
    let time_map = match &mapping.time_map {
        SourceTimeMap::Linear {
            source_start,
            rate,
            repeat,
            direction,
        } => ResolvedSourceTimeMap::Linear {
            source_range_per_repeat: time::source_range(
                *source_start,
                clip.record_range.duration,
                *rate,
                *repeat,
            )?,
            rate: *rate,
            repeat: *repeat,
            direction: *direction,
        },
        SourceTimeMap::Curve { segments } => ResolvedSourceTimeMap::Curve {
            segments: segments.clone(),
        },
    };
    Some(ResolvedSourceMapping {
        time_map,
        frame_synthesis: mapping.frame_synthesis,
        out_of_range: mapping.out_of_range,
    })
}

struct CurveBounds {
    start: veac_ir::RationalTime,
    end: veac_ir::RationalTime,
    end_is_point: bool,
}

fn curve_bounds(segments: &[veac_ir::SourceTimeSegment]) -> Option<CurveBounds> {
    let first = segments.first()?;
    let (mut start, mut end) = ordered(first.source_start, first.source_end);
    let mut end_is_point = first.source_start == end;
    for segment in segments {
        for (value, is_point) in [(segment.source_start, true), (segment.source_end, false)] {
            if value < start {
                start = value;
            }
            if value > end {
                end = value;
                end_is_point = is_point;
            } else if value == end {
                end_is_point |= is_point;
            }
        }
    }
    Some(CurveBounds {
        start,
        end,
        end_is_point,
    })
}

fn ordered(
    left: veac_ir::RationalTime,
    right: veac_ir::RationalTime,
) -> (veac_ir::RationalTime, veac_ir::RationalTime) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}
