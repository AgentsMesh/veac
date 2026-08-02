mod support;

use support::*;
use veac_plan::{canonical::*, resolve, ResolvedSourceTimeMap};

#[test]
fn source_time_curve_is_owned_by_the_plan_without_rate_flattening() {
    let mut project = project();
    project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(curve());
    let plan = resolve(&project, None).unwrap().remove(0);
    let mapping = plan.sequences[0].tracks[0].clips[0]
        .source_mapping
        .as_ref()
        .unwrap();
    let ResolvedSourceTimeMap::Curve { segments } = &mapping.time_map else {
        panic!("curve mapping");
    };
    assert_eq!(segments.len(), 3);
    assert_eq!(segments[0].source_end, time(300));
    assert_eq!(segments[1].interpolation, SourceTimeInterpolation::Hold);
    assert_eq!(mapping.frame_synthesis, FrameSynthesisPolicy::Blend);
}

#[test]
fn curve_and_hold_source_bounds_fail_before_codegen() {
    for mapping in [
        SourceMapping {
            time_map: SourceTimeMap::Curve {
                segments: vec![segment(600, 0, 60_000, SourceTimeInterpolation::Linear)],
            },
            frame_synthesis: FrameSynthesisPolicy::Nearest,
            out_of_range: SourceOutOfRangePolicy::Strict,
        },
        SourceMapping {
            time_map: SourceTimeMap::Curve {
                segments: vec![segment(600, 60_000, 60_000, SourceTimeInterpolation::Hold)],
            },
            frame_synthesis: FrameSynthesisPolicy::Nearest,
            out_of_range: SourceOutOfRangePolicy::Strict,
        },
    ] {
        let mut project = project();
        project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(mapping);
        let error = resolve(&project, None).unwrap_err();
        assert!(error
            .diagnostics()
            .iter()
            .any(|item| item.code == "SOURCE_RANGE_OUT_OF_BOUNDS"));
    }
}

#[test]
fn source_boundary_policies_are_directional() {
    for (start, rate, policy, valid) in [
        (-600, 1, SourceOutOfRangePolicy::Strict, false),
        (-600, 1, SourceOutOfRangePolicy::HoldFirst, true),
        (-600, 1, SourceOutOfRangePolicy::HoldLast, false),
        (-600, 1, SourceOutOfRangePolicy::HoldBoth, true),
        (5_700, 1, SourceOutOfRangePolicy::Strict, false),
        (5_700, 1, SourceOutOfRangePolicy::HoldFirst, false),
        (5_700, 1, SourceOutOfRangePolicy::HoldLast, true),
        (5_700, 1, SourceOutOfRangePolicy::HoldBoth, true),
        (-600, 12, SourceOutOfRangePolicy::HoldFirst, false),
        (-600, 12, SourceOutOfRangePolicy::HoldLast, false),
        (-600, 12, SourceOutOfRangePolicy::HoldBoth, true),
    ] {
        let mut project = project();
        project.project.sequences[0].tracks[0].clips[0].source_mapping =
            Some(linear(start, rate, policy));
        let result = resolve(&project, None);
        assert_eq!(
            result.is_ok(),
            valid,
            "{start}/{rate}/{policy:?}: {result:?}"
        );
    }
}

#[test]
fn a_curve_that_samples_the_duration_endpoint_needs_hold_last() {
    let mapping = |policy| SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![
                segment(300, 5_700, 6_000, SourceTimeInterpolation::Linear),
                segment(300, 6_000, 6_000, SourceTimeInterpolation::Hold),
            ],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: policy,
    };
    for (policy, valid) in [
        (SourceOutOfRangePolicy::Strict, false),
        (SourceOutOfRangePolicy::HoldLast, true),
    ] {
        let mut project = project();
        project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(mapping(policy));
        assert_eq!(resolve(&project, None).is_ok(), valid, "{policy:?}");
    }
}

#[test]
fn negative_hold_still_requires_a_known_source_duration() {
    let mut project = project();
    let material = &mut project.project.materials[0];
    let probe = material.probe.as_mut().unwrap();
    probe.container_duration = None;
    probe.streams[0].duration = None;
    project.project.sequences[0].tracks[0].clips[0].source_mapping = Some(SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![segment(600, -600, -600, SourceTimeInterpolation::Hold)],
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::HoldFirst,
    });
    let error = resolve(&project, None).unwrap_err();
    assert!(diagnostic_codes(&error).contains(&"SOURCE_DURATION_UNAVAILABLE"));
}

#[test]
fn a_nonzero_stream_start_disables_container_duration_fallback() {
    let mut project = project();
    let probe = project.project.materials[0].probe.as_mut().unwrap();
    probe.streams[0].duration = None;
    probe.streams[0].start_time = Some(time(300));
    let error = resolve(&project, None).unwrap_err();
    assert!(diagnostic_codes(&error).contains(&"SOURCE_DURATION_UNAVAILABLE"));
}

fn linear(start: i64, rate: i64, policy: SourceOutOfRangePolicy) -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start: time(start),
            rate: Rational::new(rate, 1).unwrap(),
            direction: PlaybackDirection::Forward,
            repeat: 1,
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: policy,
    }
}

fn curve() -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![
                segment(200, 0, 300, SourceTimeInterpolation::Linear),
                segment(100, 300, 300, SourceTimeInterpolation::Hold),
                segment(300, 300, 450, SourceTimeInterpolation::Linear),
            ],
        },
        frame_synthesis: FrameSynthesisPolicy::Blend,
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
