use veac_plan::canonical::{
    DeliverableKind, ItemId, MaterialKind, PlaybackDirection, Rational, TrackId, VideoCadence,
};
use veac_plan::{ResolvedInputKind, ResolvedSourceTimeMap};

use super::support::*;

#[test]
fn one_reverse_succeeds_and_non_overlapping_branches_accumulate() {
    let mut plan = resolved(&fixture());
    reverse(&mut plan);
    assert!(codes(&plan).is_empty());

    let mut second = plan.sequences[0].tracks[0].clone();
    second.id = TrackId::new("trk_reverse_second").unwrap();
    second.order = 1;
    second.source_order = 1;
    second.clips[0].id = ItemId::new("itm_reverse_second").unwrap();
    second.clips[0].record_range.start = time(600);
    plan.sequences[0].tracks.push(second);
    plan.sequences[0].duration = time(1_200);
    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);
}

#[test]
fn every_descending_curve_segment_is_charged_but_repeat_reuses_one_buffer() {
    let mut curve = resolved(&fixture());
    reverse(&mut curve).time_map = ResolvedSourceTimeMap::Curve {
        segments: vec![segment(1_200, 600), segment(600, 0)],
    };
    curve.sequences[0].tracks[0].clips[0].record_range.duration = time(1_200);
    curve.sequences[0].duration = time(1_200);
    assert_eq!(codes(&curve), ["PLAN_BUDGET_REVERSE_BYTES"]);

    let mut repeat = resolved(&fixture());
    let mapping = reverse(&mut repeat);
    if let ResolvedSourceTimeMap::Linear { repeat, .. } = &mut mapping.time_map {
        *repeat = 2;
    }
    repeat.sequences[0].tracks[0].clips[0].record_range.duration = time(1_200);
    repeat.sequences[0].duration = time(1_200);
    assert!(codes(&repeat).is_empty());
}

#[test]
fn effective_proxy_video_facts_replace_original_facts() {
    let mut plan = resolved(&fixture());
    reverse(&mut plan);
    let small = proxy_video_bindings(&plan, 640, 360, 30);
    let large = proxy_video_bindings(&plan, 3_840, 2_160, 120);
    let info = &mut plan.inputs[0].video.as_mut().unwrap().info;
    info.width = 320;
    info.height = 180;
    info.cadence = VideoCadence::Variable;

    assert!(codes_with(&plan, &small).is_empty());
    assert_eq!(codes_with(&plan, &large), ["PLAN_BUDGET_REVERSE_BYTES"]);
    assert_eq!(codes(&plan), ["PLAN_REVERSE_CADENCE_UNSUPPORTED"]);
}

#[test]
fn reverse_fails_closed_for_missing_facts_or_non_constant_cadence() {
    for cadence in [VideoCadence::Unknown, VideoCadence::Variable] {
        let mut plan = resolved(&fixture());
        reverse(&mut plan);
        plan.inputs[0].video.as_mut().unwrap().info.cadence = cadence;
        assert_eq!(codes(&plan), ["PLAN_REVERSE_CADENCE_UNSUPPORTED"]);
    }
}

#[test]
fn absent_facts_fail_at_the_emission_wrapper_and_images_skip_it() {
    let mut plan = resolved(&fixture());
    reverse(&mut plan);
    let bindings = crate::unit_tests::emitter_tests::support::bindings(&plan);
    let deliverable = &plan.output.deliverables[0];
    let alpha = match &deliverable.kind {
        DeliverableKind::Video(video) => video.video.alpha,
        _ => unreachable!(),
    };
    let mut context =
        crate::emitter::EmitContext::new_visual(&plan, &bindings, deliverable, alpha).unwrap();
    let error = context
        .reverse_video(
            "raw",
            "reversev",
            &plan.sequences[0].tracks[0].clips[0],
            time(1),
            None,
        )
        .unwrap_err();
    assert_eq!(
        error.diagnostics()[0].code,
        "PLAN_REVERSE_SOURCE_FACTS_MISSING"
    );

    plan.inputs[0].kind = ResolvedInputKind::Media {
        material_kind: MaterialKind::Image,
    };
    plan.inputs[0].audio = None;
    assert!(codes(&plan).is_empty());
}

#[test]
fn exact_budget_boundary_uses_source_raster_and_rate() {
    let mut plan = resolved(&fixture());
    reverse(&mut plan);
    let info = &mut plan.inputs[0].video.as_mut().unwrap().info;
    info.width = 3_840;
    info.height = 2_160;
    info.frame_rate = Some(Rational::new(120, 1).unwrap());
    assert_eq!(codes(&plan), ["PLAN_BUDGET_REVERSE_BYTES"]);

    if let ResolvedSourceTimeMap::Linear { direction, .. } = &mut plan.sequences[0].tracks[0].clips
        [0]
    .source_mapping
    .as_mut()
    .unwrap()
    .time_map
    {
        *direction = PlaybackDirection::Forward;
    }
    assert!(codes(&plan).is_empty());
}
