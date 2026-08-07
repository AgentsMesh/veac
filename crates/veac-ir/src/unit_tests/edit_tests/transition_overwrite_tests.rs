use crate::test_support::add_transition;

use super::*;

#[test]
fn overwrite_preserves_only_relations_on_surviving_outer_edges() {
    let mut project = video_project(&[
        ("itm_p", 0, 100),
        ("itm_a", 80, 100),
        ("itm_b", 160, 100),
        ("itm_c", 240, 100),
        ("itm_d", 320, 100),
    ]);
    for (from, to) in [
        ("itm_p", "itm_a"),
        ("itm_a", "itm_b"),
        ("itm_b", "itm_c"),
        ("itm_c", "itm_d"),
    ] {
        add_transition(&mut project, "seq_main", from, to, dissolve());
    }
    let result = overwrite(&project, 130, 160, vec![]);
    assert_eq!(result.project.relations.len(), 2);
    assert_edge(&result, "rel_transition_0", "itm_p", "itm_a");
    assert_edge(&result, "rel_transition_3", "itm_c", "itm_d");
}

#[test]
fn spanning_overwrite_moves_outgoing_to_the_right_fragment() {
    let mut project = video_project(&[
        ("itm_p", 0, 100),
        ("itm_span", 80, 300),
        ("itm_d", 360, 100),
    ]);
    add_transition(&mut project, "seq_main", "itm_p", "itm_span", dissolve());
    add_transition(&mut project, "seq_main", "itm_span", "itm_d", dissolve());
    let result = overwrite(
        &project,
        200,
        100,
        vec![OverwriteFragment {
            source_clip_id: id("itm_span"),
            right_fragment_id: id("itm_span_right"),
            relation_fragments: vec![],
        }],
    );
    assert_edge(&result, "rel_transition_0", "itm_p", "itm_span");
    assert_edge(&result, "rel_transition_1", "itm_span_right", "itm_d");
}

fn overwrite(
    project: &ProjectEnvelope,
    start: i64,
    duration: i64,
    fragments: Vec<OverwriteFragment>,
) -> ProjectEnvelope {
    let mut replacement = project.project.sequences[0].tracks[0].clips[0].clone();
    replacement.id = id("itm_replacement");
    replacement.record_range = range(start, duration);
    applied(apply_edit_batch(
        project,
        &batch(
            "op_transition_overwrite",
            project,
            vec![EditOperation::OverwriteClip {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                track_id: TrackId::new("trk_video").unwrap(),
                clip: Box::new(replacement),
                split_fragments: fragments,
            }],
        ),
    ))
}

fn video_project(spec: &[(&str, i64, i64)]) -> ProjectEnvelope {
    let mut project = sample_project();
    let track = &mut project.project.sequences[0].tracks[0];
    track.placement_mode = PlacementMode::Free;
    let prototype = track.clips[0].clone();
    track.clips = spec
        .iter()
        .map(|(id, start, duration)| {
            let mut clip = prototype.clone();
            clip.id = ItemId::new(*id).unwrap();
            clip.record_range = range(*start, *duration);
            clip.effects.clear();
            clip.visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
            clip
        })
        .collect();
    project
}

fn dissolve() -> Transition {
    Transition {
        kind: TransitionKind::Dissolve,
        duration: time(20),
        alignment: TransitionAlignment::Centered,
    }
}

fn id(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

fn assert_edge(project: &ProjectEnvelope, relation: &str, from: &str, to: &str) {
    let relation = project
        .project
        .relations
        .iter()
        .find(|value| value.id.as_str() == relation)
        .unwrap();
    let RelationKind::Transition { from: a, to: b, .. } = &relation.kind else {
        panic!("expected transition")
    };
    assert_eq!(a.item_id().unwrap().as_str(), from);
    assert_eq!(b.item_id().unwrap().as_str(), to);
}
