use crate::test_support::{add_transition, identity_layout_visual, range, time, transition_mut};

use super::*;

#[test]
fn exact_overlap_and_odd_tick_duration_are_canonical() {
    validate(&transition_project(60)).unwrap();
    validate(&transition_project(61)).unwrap();
}

#[test]
fn touching_gap_and_containment_are_not_transitions() {
    for range in [range(600, 600), range(601, 600), range(300, 100)] {
        let mut project = transition_project(60);
        project.project.sequences[0].tracks[0].clips[1].record_range = range;
        assert_code(&validation_codes(&project), "TRANSITION_OVERLAP");
    }
}

#[test]
fn authored_duration_must_equal_the_exact_intersection() {
    let mut project = transition_project(60);
    transition_mut(&mut project, "itm_video").duration = time(59);
    assert_code(&validation_codes(&project), "TRANSITION_DURATION_MISMATCH");
}

#[test]
fn transition_requires_visual_endpoints() {
    let mut audio = transition_project(60);
    audio.project.sequences[0].tracks[0].kind = TrackKind::Audio;
    assert_code(&validation_codes(&audio), "TRANSITION_TRACK_TYPE");

    let mut missing = transition_project(60);
    missing.project.sequences[0].tracks[0].clips[1].visual = None;
    assert_code(&validation_codes(&missing), "TRANSITION_VISUAL_ENDPOINT");
}

#[test]
fn transition_alignment_is_a_centered_only_json_contract() {
    let project = transition_project(60);
    let canonical = canonical_json(&project).unwrap();
    assert!(canonical.contains(r#""alignment":"centered""#));
    assert_eq!(decode_canonical_json(&canonical).unwrap(), project);

    for legacy in ["before_cut", "after_cut"] {
        let mut value = serde_json::to_value(&project).unwrap();
        value["project"]["relations"][0]["kind"]["transition"]["alignment"] =
            serde_json::json!(legacy);
        let error = decode_canonical_json(&value.to_string()).unwrap_err();
        assert!(error.to_string().contains("unknown variant"), "{error}");
    }

    let schema = project_json_schema().unwrap();
    assert_eq!(
        schema["$defs"]["TransitionAlignment"]["enum"],
        serde_json::json!(["centered"])
    );
}

#[test]
fn endpoints_must_be_same_track_adjacent_and_distinct() {
    let mut cross_track = transition_project(60);
    let target = cross_track.project.sequences[0].tracks[0]
        .clips
        .pop()
        .unwrap();
    cross_track.project.sequences[0].tracks[1]
        .clips
        .push(target);
    assert_code(&validation_codes(&cross_track), "RELATION_CONTEXT");

    let mut nonadjacent = transition_project(60);
    let mut middle = nonadjacent.project.sequences[0].tracks[0].clips[1].clone();
    middle.id = ItemId::new("itm_between").unwrap();
    middle.record_range = range(520, 10);
    nonadjacent.project.sequences[0].tracks[0]
        .clips
        .insert(1, middle);
    assert_code(&validation_codes(&nonadjacent), "RELATION_CONTEXT");

    let mut same = transition_project(60);
    let RelationKind::Transition { to, .. } = &mut same.project.relations[0].kind else {
        unreachable!()
    };
    *to = RelationEndpoint::item(ItemId::new("itm_video").unwrap());
    assert_code(&validation_codes(&same), "RELATION_CONTEXT");
}

#[test]
fn no_third_item_may_cross_the_overlap() {
    let mut project = transition_project(60);
    let mut third = project.project.sequences[0].tracks[0].clips[1].clone();
    third.id = ItemId::new("itm_crossing").unwrap();
    third.record_range = range(570, 600);
    project.project.sequences[0].tracks[0].clips.push(third);
    assert_code(&validation_codes(&project), "TRANSITION_THIRD_ITEM");

    project.project.sequences[0].tracks[0].clips[2].enabled = false;
    assert_code(&validation_codes(&project), "TRANSITION_THIRD_ITEM");
}

#[test]
fn adjacent_transition_windows_cannot_overlap_on_middle_clip() {
    let mut project = three_clip_project();
    project.project.sequences[0].tracks[0].clips[2].record_range = range(590, 600);
    transition_mut(&mut project, "itm_middle").duration = time(550);
    assert_code(&validation_codes(&project), "TRANSITION_WINDOW_OVERLAP");
}

#[test]
fn transition_blend_and_z_contracts_fail_closed() {
    let mut blend = transition_project(60);
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

fn transition_project(duration: i64) -> ProjectEnvelope {
    let mut project = sample_project();
    let track = &mut project.project.sequences[0].tracks[0];
    let mut first = track.clips[0].clone();
    first.visual = Some(identity_layout_visual());
    first.audio = None;
    first.effects.clear();
    let mut incoming = first.clone();
    incoming.id = ItemId::new("itm_middle").unwrap();
    incoming.record_range = range(600 - duration, 600);
    track.clips = vec![first, incoming];
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_middle",
        dissolve(duration),
    );
    project
}

fn three_clip_project() -> ProjectEnvelope {
    let mut project = transition_project(60);
    let mut third = project.project.sequences[0].tracks[0].clips[1].clone();
    third.id = ItemId::new("itm_third").unwrap();
    third.record_range = range(1_080, 600);
    project.project.sequences[0].tracks[0].clips.push(third);
    add_transition(
        &mut project,
        "seq_main",
        "itm_middle",
        "itm_third",
        dissolve(60),
    );
    project
}

fn dissolve(duration: i64) -> Transition {
    Transition {
        kind: TransitionKind::Dissolve,
        duration: time(duration),
        alignment: TransitionAlignment::Centered,
    }
}
