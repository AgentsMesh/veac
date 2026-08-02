use crate::test_support::time;

use super::*;

#[test]
fn monotonic_ramp_and_hold_are_valid_canonical_source_time() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(curve());
    validate(&project).unwrap();
}

#[test]
fn source_time_curve_rejects_empty_duration_continuity_direction_and_shape() {
    let mut empty = sample_project();
    empty.project.sequences[0].tracks[0].clips[0].source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Curve { segments: vec![] },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    });
    assert_code(&validation_codes(&empty), "SOURCE_TIME_MAP_EMPTY");

    for (index, mutate, code) in [
        (
            0,
            mutate_duration as fn(&mut [SourceTimeSegment]),
            "SOURCE_TIME_MAP_DURATION",
        ),
        (1, mutate_continuity, "SOURCE_TIME_MAP_CONTINUITY"),
        (2, mutate_direction, "SOURCE_TIME_MAP_MONOTONIC"),
        (3, mutate_shape, "SOURCE_TIME_INTERPOLATION"),
    ] {
        let mut project = sample_project();
        let mut mapping = curve();
        let SourceTimeMap::Curve { segments } = &mut mapping.time_map else {
            unreachable!()
        };
        mutate(segments);
        project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(mapping);
        let codes = validation_codes(&project);
        assert_code(&codes, code);
        assert!(!codes.is_empty(), "mutation {index} must fail");
    }
}

#[test]
fn linear_source_time_rejects_unbounded_repeat_fanout() {
    let mut project = sample_project();
    let Some(SourceMapping {
        time_map: SourceTimeMap::Linear { repeat, .. },
        ..
    }) = &mut project.project.sequences[0].tracks[0].clips[0].source_mapping
    else {
        panic!("linear fixture")
    };
    *repeat = MAX_SOURCE_REPEAT + 1;

    assert_code(&validation_codes(&project), "SOURCE_REPEAT");
}

#[test]
fn negative_linear_source_requires_a_first_boundary_policy() {
    for (policy, valid) in [
        (SourceOutOfRangePolicy::Strict, false),
        (SourceOutOfRangePolicy::HoldFirst, true),
        (SourceOutOfRangePolicy::HoldLast, false),
        (SourceOutOfRangePolicy::HoldBoth, true),
    ] {
        let mut project = sample_project();
        let mapping = project.project.sequences[0].tracks[0].clips[0]
            .source_mapping
            .as_mut()
            .unwrap();
        let SourceTimeMap::Linear { source_start, .. } = &mut mapping.time_map else {
            panic!("linear fixture")
        };
        *source_start = time(-100);
        mapping.out_of_range = policy;
        let result = validate(&project);
        assert_eq!(result.is_ok(), valid, "{policy:?}: {result:?}");
    }
}

#[test]
fn trailing_source_boundary_policy_is_explicit() {
    for (policy, allowed) in [
        (SourceOutOfRangePolicy::Strict, false),
        (SourceOutOfRangePolicy::HoldFirst, false),
        (SourceOutOfRangePolicy::HoldLast, true),
        (SourceOutOfRangePolicy::HoldBoth, true),
    ] {
        assert_eq!(policy.allows_after(), allowed, "{policy:?}");
    }
}

#[test]
fn negative_curve_source_uses_the_same_boundary_policy() {
    for (policy, valid) in [
        (SourceOutOfRangePolicy::Strict, false),
        (SourceOutOfRangePolicy::HoldFirst, true),
    ] {
        let mut project = sample_project();
        let mut mapping = curve();
        let SourceTimeMap::Curve { segments } = &mut mapping.time_map else {
            unreachable!()
        };
        for segment in segments {
            segment.source_start.value -= 200;
            segment.source_end.value -= 200;
        }
        mapping.out_of_range = policy;
        project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(mapping);
        let result = validate(&project);
        assert_eq!(result.is_ok(), valid, "{policy:?}: {result:?}");
    }
}

fn curve() -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![
                segment(200, 0, 100, SourceTimeInterpolation::Linear),
                segment(100, 100, 100, SourceTimeInterpolation::Hold),
                segment(300, 100, 400, SourceTimeInterpolation::Linear),
            ],
        },
        frame_synthesis: FrameSynthesisPolicy::MotionCompensated,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

fn segment(
    duration: i64,
    start: i64,
    end: i64,
    interpolation: SourceTimeInterpolation,
) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation,
    }
}

fn mutate_duration(segments: &mut [SourceTimeSegment]) {
    segments[0].record_duration = time(199);
}

fn mutate_continuity(segments: &mut [SourceTimeSegment]) {
    segments[1].source_start = time(99);
}

fn mutate_direction(segments: &mut [SourceTimeSegment]) {
    segments[2].source_end = time(50);
}

fn mutate_shape(segments: &mut [SourceTimeSegment]) {
    segments[1].interpolation = SourceTimeInterpolation::Linear;
}
