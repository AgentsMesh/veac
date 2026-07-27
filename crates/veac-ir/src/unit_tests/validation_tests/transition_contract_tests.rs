use crate::test_support::{add_transition, identity_layout_visual, range, time, transition_mut};

use super::*;

#[test]
fn transition_blend_and_z_contracts_fail_closed() {
    let mut blend = transition_project();
    blend.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .compositing = Compositing {
        z_index: 0,
        blend_mode: BlendMode::Multiply,
    };
    assert_code(&validation_codes(&blend), "TRANSITION_BLEND_MODE");

    let visual = blend.project.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap();
    visual.compositing.blend_mode = BlendMode::Normal;
    visual.compositing.z_index = 1;
    assert_code(&validation_codes(&blend), "TRANSITION_Z_ORDER");
}

#[test]
fn consecutive_transition_windows_cannot_overlap() {
    let mut project = transition_project();
    *transition_mut(&mut project, "itm_video") = transition(TransitionAlignment::AfterCut, 600);
    add_transition(
        &mut project,
        "seq_main",
        "itm_transition_middle",
        "itm_transition_third",
        transition(TransitionAlignment::BeforeCut, 600),
    );
    assert_code(&validation_codes(&project), "TRANSITION_WINDOW_OVERLAP");
}

fn transition_project() -> ProjectEnvelope {
    let mut project = sample_project();
    let track = &mut project.project.sequences[0].tracks[0];
    let mut first = track.clips[0].clone();
    first.visual = Some(identity_layout_visual());
    first.audio = None;
    first.effects.clear();
    let mut middle = first.clone();
    middle.id = ItemId::new("itm_transition_middle").unwrap();
    middle.record_range = range(600, 600);
    middle.visual = None;
    let mut third = middle.clone();
    third.id = ItemId::new("itm_transition_third").unwrap();
    third.record_range = range(1_200, 600);
    track.clips = vec![first, middle, third];
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_transition_middle",
        transition(TransitionAlignment::Centered, 60),
    );
    project
}

fn transition(alignment: TransitionAlignment, duration: i64) -> Transition {
    Transition {
        kind: TransitionKind::Dissolve,
        duration: time(duration),
        alignment,
    }
}
