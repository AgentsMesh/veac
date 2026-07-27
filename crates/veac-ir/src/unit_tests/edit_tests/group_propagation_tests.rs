use super::*;

#[test]
fn membership_graph_propagates_across_clip_group_and_av_link_edges() {
    let mut project = linked_project();
    project.project.relations.push(Relation {
        id: RelationId::new("rel_scene").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Group {
            members: vec![
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_video").unwrap(),
                },
                RelationEndpoint::Item {
                    item_id: ItemId::new("itm_caption").unwrap(),
                },
            ],
        },
    });
    let edit = batch(
        "op_connected_graph",
        &project,
        vec![
            EditOperation::MoveClip {
                clip_id: ItemId::new("itm_audio").unwrap(),
                record_start: time(100),
            },
            EditOperation::TrimClip {
                clip_id: ItemId::new("itm_caption").unwrap(),
                edge: TrimEdge::Out,
                delta: time(-60),
                ripple: false,
            },
            EditOperation::SetClipEnabled {
                clip_id: ItemId::new("itm_video").unwrap(),
                enabled: false,
            },
        ],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let sequence = &result.project.sequences[0];
    for id in ["itm_video", "itm_audio", "itm_caption"] {
        let clip = sequence
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .find(|clip| clip.id.as_str() == id)
            .unwrap();
        assert_eq!(clip.record_range.start, time(100));
        assert!(!clip.enabled);
    }
    assert_eq!(sequence.tracks[0].clips[0].record_range.duration, time(540));
    assert_eq!(sequence.tracks[1].clips[0].record_range.duration, time(540));
    assert_eq!(sequence.tracks[2].clips[0].record_range.duration, time(240));

    let delete = batch(
        "op_delete_connected_graph",
        &result,
        vec![EditOperation::RemoveClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
        }],
    );
    let deleted = applied(apply_edit_batch(&result, &delete));
    let sequence = &deleted.project.sequences[0];
    assert!(deleted.project.relations.iter().all(|relation| !matches!(
        relation.kind,
        RelationKind::Group { .. } | RelationKind::AvLink { .. }
    )));
    assert!(sequence.tracks.iter().all(|track| track.clips.is_empty()));
}

#[test]
fn ripple_delete_removes_linked_members_and_the_link_record() {
    let project = linked_project();
    let edit = batch(
        "op_ripple_delete_link",
        &project,
        vec![EditOperation::RippleDelete {
            clip_id: ItemId::new("itm_video").unwrap(),
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
fn overwrite_rejects_relationship_members_instead_of_silently_severing_them() {
    let project = linked_project();
    let mut replacement = project.project.sequences[0].tracks[0].clips[0].clone();
    replacement.id = ItemId::new("itm_overwrite_link").unwrap();
    replacement.record_range = range(100, 100);
    replacement.effects.clear();
    replacement.visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    let edit = batch(
        "op_overwrite_link",
        &project,
        vec![EditOperation::OverwriteClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_video").unwrap(),
            clip: Box::new(replacement),
            split_fragments: vec![],
        }],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
}
