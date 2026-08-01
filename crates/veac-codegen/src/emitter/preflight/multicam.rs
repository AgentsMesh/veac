use std::collections::BTreeSet;

use veac_plan::canonical::{MulticamAngleId, MulticamGroupId, RationalTime};
use veac_plan::{ResolvedClip, ResolvedClipSource, ResolvedMulticamSource, ResolvedRenderPlan};

use super::{input_streams, source_time, Check};

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    for clip in plan
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
    {
        let ResolvedClipSource::Multicam { source } = &clip.source else {
            continue;
        };
        if !valid(plan, clip, source) {
            check.push(
                "PLAN_MULTICAM_INVALID",
                Some(clip.id.to_string()),
                "multicam identity, angles, switch partition, or source bounds are invalid",
            );
        }
    }
}

fn valid(plan: &ResolvedRenderPlan, clip: &ResolvedClip, source: &ResolvedMulticamSource) -> bool {
    if MulticamGroupId::new(source.group_id.as_str()).is_err()
        || source.angles.is_empty()
        || !source.angles.windows(2).all(|pair| pair[0].id < pair[1].id)
    {
        return false;
    }
    let mut ids = BTreeSet::new();
    if source.angles.iter().any(|angle| {
        MulticamAngleId::new(angle.id.as_str()).is_err()
            || !ids.insert(angle.id.to_string())
            || !local_point(angle.source_offset, plan.header.source.timebase)
            || input_streams::trusted_angle(plan, angle, clip.audio.is_some()).is_none()
    }) || MulticamAngleId::new(source.sync.reference_angle_id.as_str()).is_err()
    {
        return false;
    }
    let mut expected = RationalTime {
        value: 0,
        timescale: plan.header.source.timebase,
    };
    let mut switched = BTreeSet::new();
    for switch in &source.switches {
        let Some(angle) = source
            .angles
            .iter()
            .find(|angle| angle.id == switch.angle_id)
        else {
            return false;
        };
        switched.insert(switch.angle_id.to_string());
        if switch.range.start != expected
            || !local_duration(switch.range.duration, plan.header.source.timebase)
        {
            return false;
        }
        let Some(end) = switch.range.end().ok() else {
            return false;
        };
        let Some(source_end) = angle.source_offset.checked_add(end).ok() else {
            return false;
        };
        let Some(input) = input_streams::trusted_angle(plan, angle, clip.audio.is_some()) else {
            return false;
        };
        if !source_time::input_bounds(input, source_end, true, clip.audio.is_some()) {
            return false;
        }
        expected = end;
    }
    !source.switches.is_empty() && expected == clip.record_range.duration && switched == ids
}

fn local_point(value: RationalTime, timebase: u32) -> bool {
    value.is_valid() && value.timescale == timebase && value.value >= 0
}

fn local_duration(value: RationalTime, timebase: u32) -> bool {
    value.is_valid() && value.timescale == timebase && value.value > 0
}
