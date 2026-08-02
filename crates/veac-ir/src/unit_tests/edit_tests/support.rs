use crate::{Clip, PlaybackDirection, Rational, RationalTime, SourceTimeMap};

pub(super) fn linear_map_mut(
    clip: &mut Clip,
) -> (
    &mut RationalTime,
    &mut Rational,
    &mut u32,
    &mut PlaybackDirection,
) {
    let SourceTimeMap::Linear {
        source_start,
        rate,
        repeat,
        direction,
    } = &mut clip
        .source_mapping
        .as_mut()
        .expect("media source mapping")
        .time_map
    else {
        panic!("expected linear source mapping");
    };
    (source_start, rate, repeat, direction)
}

pub(super) fn set_linear_start(clip: &mut Clip, value: RationalTime) {
    let SourceTimeMap::Linear { source_start, .. } = &mut clip
        .source_mapping
        .as_mut()
        .expect("media source mapping")
        .time_map
    else {
        panic!("expected linear source mapping");
    };
    *source_start = value;
}

pub(super) fn linear_start(clip: &Clip) -> RationalTime {
    let SourceTimeMap::Linear { source_start, .. } = &clip
        .source_mapping
        .as_ref()
        .expect("media source mapping")
        .time_map
    else {
        panic!("expected linear source mapping");
    };
    *source_start
}
