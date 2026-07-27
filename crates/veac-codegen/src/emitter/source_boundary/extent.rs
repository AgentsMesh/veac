use std::cmp::Ordering;

use veac_plan::canonical::{RationalTime, SourceTimeInterpolation, SourceTimeSegment};
use veac_plan::ResolvedSourceTimeMap;

pub(super) struct Extent {
    pub minimum: RationalTime,
    pub maximum: RationalTime,
    pub maximum_is_point: bool,
}

pub(super) fn mapping(mapping: &ResolvedSourceTimeMap, include_points: bool) -> Option<Extent> {
    match mapping {
        ResolvedSourceTimeMap::Linear {
            source_range_per_repeat,
            ..
        } => Some(Extent {
            minimum: source_range_per_repeat.start,
            maximum: source_range_per_repeat.end().ok()?,
            maximum_is_point: false,
        }),
        ResolvedSourceTimeMap::Curve { segments } => curve(segments, include_points),
    }
}

fn curve(segments: &[SourceTimeSegment], include_points: bool) -> Option<Extent> {
    let mut values = segments
        .iter()
        .filter(|segment| include_points || segment.interpolation != SourceTimeInterpolation::Hold)
        .flat_map(|segment| {
            [
                (segment.source_start, include_points),
                (segment.source_end, false),
            ]
        });
    let (first, first_is_point) = values.next()?;
    let mut extent = Extent {
        minimum: first,
        maximum: first,
        maximum_is_point: first_is_point,
    };
    for (value, is_point) in values {
        if value.partial_cmp(&extent.minimum)? == Ordering::Less {
            extent.minimum = value;
        }
        match value.partial_cmp(&extent.maximum)? {
            Ordering::Greater => {
                extent.maximum = value;
                extent.maximum_is_point = is_point;
            }
            Ordering::Equal => extent.maximum_is_point |= is_point,
            Ordering::Less => {}
        }
    }
    Some(extent)
}
