mod audio_rate;
mod bounds;
mod mapping;

use veac_plan::canonical::{RationalTime, SourceOutOfRangePolicy};
use veac_plan::{ResolvedClip, ResolvedClipSource, ResolvedRenderPlan};

use super::{input_streams, Check};

#[derive(Clone, Copy)]
pub(super) enum Extent {
    Point(RationalTime),
    Range {
        start: RationalTime,
        end: RationalTime,
        upper_inclusive: bool,
    },
}

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    audio_rate::validate(check, plan);
    for clip in plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
    {
        match &clip.source {
            ResolvedClipSource::FreezeFrame { source_time, .. } => {
                freeze(check, plan, clip, *source_time)
            }
            ResolvedClipSource::Media { .. } | ResolvedClipSource::Sequence { .. } => {
                let Some(mapping) = clip.source_mapping.as_ref() else {
                    bounds_error(check, clip);
                    continue;
                };
                let extent = mapping::extent(
                    &mapping.time_map,
                    mapping.out_of_range,
                    clip,
                    plan.header.source.timebase,
                );
                if extent.is_none() {
                    check.push(
                        "PLAN_SOURCE_TIME_MAP_INVALID",
                        Some(clip.id.to_string()),
                        "source-time mapping violates canonical duration, shape, or arithmetic rules",
                    );
                }
                if let (Some(extent), Some(input)) =
                    (extent, input_streams::trusted_media(plan, clip))
                {
                    if let ResolvedClipSource::Media {
                        video_stream,
                        audio_stream,
                        ..
                    } = &clip.source
                    {
                        if !bounds::valid(
                            input,
                            extent,
                            video_stream.is_some(),
                            audio_stream.is_some(),
                            mapping.out_of_range,
                        ) {
                            bounds_error(check, clip);
                        }
                    }
                }
                if let (Some(extent), ResolvedClipSource::Sequence { sequence_id }) =
                    (extent, &clip.source)
                {
                    let valid = plan
                        .sequences
                        .iter()
                        .find(|sequence| sequence.id == *sequence_id)
                        .is_some_and(|sequence| {
                            bounds::valid_duration(sequence.duration, extent, mapping.out_of_range)
                        });
                    if !valid {
                        bounds_error(check, clip);
                    }
                }
            }
            _ => {}
        }
    }
}

pub(super) fn input_bounds(
    input: &veac_plan::ResolvedInput,
    end: RationalTime,
    video: bool,
    audio: bool,
) -> bool {
    let Ok(start) = RationalTime::zero(end.timescale) else {
        return false;
    };
    bounds::valid(
        input,
        Extent::Range {
            start,
            end,
            upper_inclusive: false,
        },
        video,
        audio,
        SourceOutOfRangePolicy::Strict,
    )
}

fn freeze(
    check: &mut Check,
    plan: &ResolvedRenderPlan,
    clip: &ResolvedClip,
    source_time: RationalTime,
) {
    if !mapping::valid_point(source_time, plan.header.source.timebase) {
        check.push(
            "PLAN_SOURCE_TIME_INVALID",
            Some(clip.id.to_string()),
            "freeze source time must be nonnegative, safe, and use the project timebase",
        );
        return;
    }
    if let Some(input) = input_streams::trusted_media(plan, clip) {
        if !bounds::valid(
            input,
            Extent::Point(source_time),
            true,
            false,
            SourceOutOfRangePolicy::Strict,
        ) {
            bounds_error(check, clip);
        }
    }
}

fn bounds_error(check: &mut Check, clip: &ResolvedClip) {
    check.push(
        "PLAN_SOURCE_TIME_BOUNDS_INVALID",
        Some(clip.id.to_string()),
        "source-time mapping exceeds the selected stream duration",
    );
}
