use std::cmp::Ordering;

use veac_plan::canonical::{MaterialKind, RationalTime, SourceOutOfRangePolicy};
use veac_plan::{ResolvedInput, ResolvedInputKind};

use super::Extent;

pub(super) fn valid(
    input: &ResolvedInput,
    extent: Extent,
    video: bool,
    audio: bool,
    outside: SourceOutOfRangePolicy,
) -> bool {
    let image = matches!(
        input.kind,
        ResolvedInputKind::Media {
            material_kind: MaterialKind::Image
        } | ResolvedInputKind::Resource {
            material_kind: MaterialKind::Image
        }
    );
    let container = input
        .probe
        .as_ref()
        .and_then(|probe| probe.container_duration);
    (!video
        || image
        || bound(
            extent,
            input
                .video
                .as_ref()
                .and_then(|item| stream_duration(item.duration, item.start_time, container)),
            outside,
        ))
        && (!audio
            || bound(
                extent,
                input
                    .audio
                    .as_ref()
                    .and_then(|item| stream_duration(item.duration, item.start_time, container)),
                outside,
            ))
}

pub(super) fn valid_duration(
    duration: RationalTime,
    extent: Extent,
    outside: SourceOutOfRangePolicy,
) -> bool {
    bound(extent, Some(duration), outside)
}

fn bound(extent: Extent, duration: Option<RationalTime>, outside: SourceOutOfRangePolicy) -> bool {
    let Some(duration) = duration.filter(|value| value.is_valid() && value.value > 0) else {
        return false;
    };
    match extent {
        Extent::Point(point) => point_valid(point, duration, outside),
        Extent::Range {
            start,
            end,
            upper_inclusive,
        } => {
            (start.value >= 0 || outside.allows_before())
                && (matches!(end.partial_cmp(&duration), Some(Ordering::Less))
                    || (end.partial_cmp(&duration) == Some(Ordering::Equal) && !upper_inclusive)
                    || outside.allows_after())
        }
    }
}

fn stream_duration(
    duration: Option<RationalTime>,
    start_time: Option<RationalTime>,
    container: Option<RationalTime>,
) -> Option<RationalTime> {
    duration.or_else(|| {
        start_time
            .map(|time| time.value == 0)
            .unwrap_or(true)
            .then_some(container)
            .flatten()
    })
}

fn point_valid(
    point: RationalTime,
    duration: RationalTime,
    outside: SourceOutOfRangePolicy,
) -> bool {
    if point.value < 0 {
        outside.allows_before()
    } else {
        point.partial_cmp(&duration) == Some(Ordering::Less) || outside.allows_after()
    }
}
