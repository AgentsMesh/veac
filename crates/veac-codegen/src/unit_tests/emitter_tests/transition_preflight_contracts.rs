use veac_plan::canonical::*;

use super::support::{bindings, emit_video_command, time};
use super::transitions::transition_plan;

#[test]
fn transition_blend_and_z_order_fail_closed() {
    let mut blend = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    blend.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .compositing
        .blend_mode = BlendMode::Multiply;
    assert_code(&blend, "PLAN_TRANSITION_BLEND_UNSUPPORTED");

    let mut z_order = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    z_order.sequences[0].tracks[0].clips[1]
        .visual
        .as_mut()
        .unwrap()
        .compositing
        .z_index += 1;
    assert_code(&z_order, "PLAN_TRANSITION_Z_ORDER_UNSUPPORTED");
}

#[test]
fn transition_handle_and_sequence_window_bounds_fail_closed() {
    let mut handle = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    handle.sequences[0].tracks[0].transitions[0]
        .incoming_handle
        .duration = time(601);
    assert_code(&handle, "PLAN_TRANSITION_INVALID");

    let mut window = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    window.sequences[0].duration = time(659);
    assert_code(&window, "PLAN_TRANSITION_INVALID");
}

#[test]
fn consecutive_transition_windows_cannot_overlap_on_middle_clip() {
    let mut plan = transition_plan(TransitionAlignment::Centered, TransitionKind::Dissolve);
    let track = &mut plan.sequences[0].tracks[0];
    let mut third = track.clips[1].clone();
    third.id = ItemId::new("itm_transition_third").unwrap();
    third.source_order = 2;
    third.record_range.start = time(1_200);
    let mut second = track.transitions[0].clone();
    second.outgoing_clip_id = track.clips[1].id.clone();
    second.incoming_clip_id = third.id.clone();
    second.cut_time = time(1_200);
    second.record_window.start = time(659);
    track.clips.push(third);
    track.transitions.push(second);
    plan.sequences[0].duration = time(1_800);
    assert_code(&plan, "PLAN_TRANSITION_WINDOW_OVERLAP");
}

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, expected: &str) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == expected),
        "missing {expected}: {error}"
    );
}
