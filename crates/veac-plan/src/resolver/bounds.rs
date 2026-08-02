mod check;
mod context;
mod timing;

pub(super) use context::BoundContext;

use veac_ir::{Clip, MaterialKind, RationalTime, SourceOutOfRangePolicy, TimeRange};

use super::{material::InputUsage, PlanResolver};
use crate::{ResolvedInput, ResolvedInputKind};

impl PlanResolver<'_> {
    pub(super) fn check_source_bounds(
        &mut self,
        input: &ResolvedInput,
        range: TimeRange,
        usage: InputUsage,
        context: BoundContext<'_>,
        upper_inclusive: bool,
    ) {
        let container = input
            .probe
            .as_ref()
            .and_then(|probe| probe.container_duration);
        let image = matches!(
            &input.kind,
            ResolvedInputKind::Media {
                material_kind: MaterialKind::Image
            }
        );
        if usage.video && !image {
            if let Some(stream) = &input.video {
                self.check_range_duration(
                    range,
                    timing::stream_duration(stream.duration, stream.start_time, container),
                    context,
                    "video",
                    upper_inclusive,
                );
            }
        }
        if usage.audio {
            if let Some(stream) = &input.audio {
                self.check_range_duration(
                    range,
                    timing::stream_duration(stream.duration, stream.start_time, container),
                    context,
                    "audio",
                    upper_inclusive,
                );
            }
        }
    }

    pub(super) fn check_freeze_bounds(
        &mut self,
        input: &ResolvedInput,
        source_time: RationalTime,
        clip: &Clip,
        path: &str,
    ) {
        self.check_source_point_bounds(
            input,
            source_time,
            InputUsage {
                video: true,
                audio: false,
                font: false,
            },
            BoundContext::new(clip, path, SourceOutOfRangePolicy::Strict),
        );
    }

    pub(super) fn check_source_point_bounds(
        &mut self,
        input: &ResolvedInput,
        source_time: RationalTime,
        usage: InputUsage,
        context: BoundContext<'_>,
    ) {
        let container = input
            .probe
            .as_ref()
            .and_then(|probe| probe.container_duration);
        let image = matches!(
            &input.kind,
            ResolvedInputKind::Media {
                material_kind: MaterialKind::Image
            }
        );
        if usage.video && !image {
            let duration = input.video.as_ref().and_then(|stream| {
                timing::stream_duration(stream.duration, stream.start_time, container)
            });
            self.check_point_duration(source_time, duration, context, "video");
        }
        if usage.audio {
            let duration = input.audio.as_ref().and_then(|stream| {
                timing::stream_duration(stream.duration, stream.start_time, container)
            });
            self.check_point_duration(source_time, duration, context, "audio");
        }
    }

    pub(super) fn check_sequence_source_bounds(
        &mut self,
        duration: RationalTime,
        range: TimeRange,
        context: BoundContext<'_>,
        upper_inclusive: bool,
    ) {
        self.check_range_duration(
            range,
            Some(duration),
            context,
            "nested sequence",
            upper_inclusive,
        );
    }

    pub(super) fn check_sequence_source_point_bounds(
        &mut self,
        duration: RationalTime,
        source_time: RationalTime,
        context: BoundContext<'_>,
    ) {
        self.check_point_duration(source_time, Some(duration), context, "nested sequence");
    }
}
