use veac_codegen::emitter::emit_all;
use veac_plan::canonical::*;
use veac_plan::{ResolvedClipSource, ResolvedSourceTimeMap};

use super::support::{bindings, fixture, resolved, time};

#[test]
fn freeze_rejects_negative_and_out_of_bounds_points_before_bundle_creation() {
    let base = resolved(&fixture());
    let input_id = base.inputs[0].id.clone();
    let selection = base.inputs[0].video.as_ref().unwrap().selection;
    for (source_time, code) in [
        (
            RationalTime::new(-1, 600).unwrap(),
            "PLAN_SOURCE_TIME_INVALID",
        ),
        (time(6_000), "PLAN_SOURCE_TIME_BOUNDS_INVALID"),
    ] {
        let mut plan = base.clone();
        clip(&mut plan).source = ResolvedClipSource::FreezeFrame {
            input_id: input_id.clone(),
            video_stream: selection,
            source_time,
        };
        assert_code(&plan, code);
    }
}

#[test]
fn malformed_linear_maps_fail_before_bundle_creation() {
    let base = resolved(&fixture());
    let mut zero_repeat = base.clone();
    let ResolvedSourceTimeMap::Linear { repeat, .. } = mapping(&mut zero_repeat) else {
        unreachable!()
    };
    *repeat = 0;
    assert_code(&zero_repeat, "PLAN_SOURCE_TIME_MAP_INVALID");

    let mut unbounded_repeat = base.clone();
    let ResolvedSourceTimeMap::Linear { repeat, .. } = mapping(&mut unbounded_repeat) else {
        unreachable!()
    };
    *repeat = MAX_SOURCE_REPEAT + 1;
    assert_code(&unbounded_repeat, "PLAN_SOURCE_TIME_MAP_INVALID");

    let mut wrong_duration = base.clone();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        ..
    } = mapping(&mut wrong_duration)
    else {
        unreachable!()
    };
    source_range_per_repeat.duration.value += 1;
    assert_code(&wrong_duration, "PLAN_SOURCE_TIME_MAP_INVALID");

    let mut mixed_timebase = base.clone();
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        ..
    } = mapping(&mut mixed_timebase)
    else {
        unreachable!()
    };
    source_range_per_repeat.start.timescale = 1_200;
    source_range_per_repeat.start.value *= 2;
    source_range_per_repeat.duration.timescale = 1_200;
    source_range_per_repeat.duration.value *= 2;
    assert_code(&mixed_timebase, "PLAN_SOURCE_TIME_MAP_INVALID");

    let mut out_of_bounds = base;
    let ResolvedSourceTimeMap::Linear {
        source_range_per_repeat,
        ..
    } = mapping(&mut out_of_bounds)
    else {
        unreachable!()
    };
    source_range_per_repeat.start = time(6_000);
    assert_code(&out_of_bounds, "PLAN_SOURCE_TIME_BOUNDS_INVALID");
}

#[test]
fn malformed_curve_shape_timebase_continuity_and_total_are_rejected() {
    let valid = vec![
        segment(300, 0, 300, SourceTimeInterpolation::Linear),
        segment(300, 300, 300, SourceTimeInterpolation::Hold),
    ];
    let mut cases = Vec::new();
    let mut discontinuous = valid.clone();
    discontinuous[1].source_start = time(301);
    cases.push(discontinuous);
    let mut bad_hold = valid.clone();
    bad_hold[1].source_end = time(301);
    cases.push(bad_hold);
    let mut bad_total = valid.clone();
    bad_total[1].record_duration = time(299);
    cases.push(bad_total);
    let mut mixed_timebase = valid;
    mixed_timebase[1].source_start = RationalTime::new(300, 1_000).unwrap();
    cases.push(mixed_timebase);

    for segments in cases {
        let mut plan = resolved(&fixture());
        *mapping(&mut plan) = ResolvedSourceTimeMap::Curve { segments };
        assert_code(&plan, "PLAN_SOURCE_TIME_MAP_INVALID");
    }
}

fn mapping(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut ResolvedSourceTimeMap {
    let mapping = clip(plan).source_mapping.as_mut().unwrap();
    &mut mapping.time_map
}

fn clip(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut veac_plan::ResolvedClip {
    &mut plan.sequences[0].tracks[0].clips[0]
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

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, code: &str) {
    let error = emit_all(plan, &bindings(plan)).unwrap_err();
    assert!(error.diagnostics().iter().any(|item| item.code == code));
}
