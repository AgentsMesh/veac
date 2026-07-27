use super::*;

#[test]
fn linked_move_trim_slip_and_enable_propagate_atomically() {
    let project = linked_project();
    let id = ItemId::new("itm_video").unwrap();
    let edit = batch(
        "op_linked_mutations",
        &project,
        vec![
            EditOperation::MoveClip {
                clip_id: id.clone(),
                record_start: time(100),
            },
            EditOperation::TrimClip {
                clip_id: id.clone(),
                edge: TrimEdge::Out,
                delta: time(-60),
                ripple: false,
            },
            EditOperation::SlipClip {
                clip_id: id.clone(),
                source_delta: time(10),
            },
            EditOperation::SetClipEnabled {
                clip_id: id,
                enabled: false,
            },
        ],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let sequence = &result.project.sequences[0];
    for id in ["itm_video", "itm_audio"] {
        let clip = sequence
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .find(|clip| clip.id.as_str() == id)
            .unwrap();
        assert_eq!(clip.record_range, range(100, 540));
        assert_eq!(linear_start(clip), time(10));
        assert!(!clip.enabled);
    }
}

#[test]
fn connected_delete_removes_all_members_and_relationship_records() {
    let project = linked_project();
    let edit = batch(
        "op_delete_linked",
        &project,
        vec![EditOperation::RemoveClip {
            clip_id: ItemId::new("itm_audio").unwrap(),
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let sequence = &result.project.sequences[0];
    assert!(result
        .project
        .relations
        .iter()
        .all(|relation| !matches!(relation.kind, RelationKind::AvLink { .. })));
    assert!(sequence.tracks[0].clips.is_empty());
    assert!(sequence.tracks[1].clips.is_empty());
}

#[test]
fn ripple_insert_shifts_linked_members_on_every_affected_track() {
    let project = linked_project();
    let mut clip = project.project.sequences[0].tracks[0].clips[0].clone();
    clip.id = ItemId::new("itm_insert_before_link").unwrap();
    clip.record_range = range(0, 60);
    clip.effects.clear();
    clip.visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    let edit = batch(
        "op_ripple_before_link",
        &project,
        vec![EditOperation::RippleInsert {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_video").unwrap(),
            clip: Box::new(clip),
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let sequence = &result.project.sequences[0];
    let starts: Vec<_> = sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .filter(|clip| matches!(clip.id.as_str(), "itm_video" | "itm_audio"))
        .map(|clip| clip.record_range.start)
        .collect();
    assert_eq!(starts, vec![time(60), time(60)]);
}

#[test]
fn locked_partner_and_relationship_unsafe_operations_reject_without_partial_mutation() {
    let mut project = linked_project();
    project.project.sequences[0].tracks[1].state.locked = true;
    let id = ItemId::new("itm_video").unwrap();
    let operations = [
        EditOperation::MoveClip {
            clip_id: id.clone(),
            record_start: time(10),
        },
        EditOperation::SplitClip {
            clip_id: id.clone(),
            at: time(300),
            right_clip_id: ItemId::new("itm_bad_split").unwrap(),
            relation_fragments: vec![],
        },
        EditOperation::SlideClip {
            clip_id: id.clone(),
            delta: time(10),
        },
    ];
    for (index, operation) in operations.into_iter().enumerate() {
        let edit = batch(&format!("op_bad_linked_{index}"), &project, vec![operation]);
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }
}
