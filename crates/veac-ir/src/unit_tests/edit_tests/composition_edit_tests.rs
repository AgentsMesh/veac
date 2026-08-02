use super::*;

#[test]
fn track_matte_is_a_typed_relation_and_invalid_reference_rolls_back() {
    let mut project = sample_project();
    let source = visual_clone(&project, "itm_matte_edit");
    project.project.sequences[0]
        .tracks
        .push(visual_track("trk_matte_edit", 5, vec![source]));
    let relation = Relation {
        id: RelationId::new("rel_matte_edit").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new("itm_matte_edit").unwrap()),
            consumer: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
            parameters: MatteRelationParameters {
                mode: TrackMatteMode::Luma,
                invert: true,
            },
        },
    };
    let edit = batch(
        "op_set_track_matte",
        &project,
        vec![EditOperation::EditStructure {
            edit: StructureEdit::InsertRelation {
                relation: relation.clone(),
            },
        }],
    );
    let updated = applied(apply_edit_batch(&project, &edit));
    assert_eq!(updated.project.relations, [relation]);

    let invalid = batch(
        "op_invalid_track_matte",
        &project,
        vec![
            EditOperation::SetClipEnabled {
                clip_id: ItemId::new("itm_caption").unwrap(),
                enabled: false,
            },
            EditOperation::EditStructure {
                edit: StructureEdit::InsertRelation {
                    relation: Relation {
                        id: RelationId::new("rel_matte_invalid").unwrap(),
                        sequence_id: SequenceId::new("seq_main").unwrap(),
                        kind: RelationKind::Matte {
                            producer: RelationEndpoint::item(ItemId::new("itm_absent").unwrap()),
                            consumer: RelationEndpoint::item(ItemId::new("itm_video").unwrap()),
                            parameters: MatteRelationParameters {
                                mode: TrackMatteMode::Alpha,
                                invert: false,
                            },
                        },
                    },
                },
            },
        ],
    );
    assert_rejected(apply_edit_batch(&project, &invalid), "EDIT_REJECTED");
    assert!(project.project.sequences[0].tracks[1].clips[0].enabled);
}

fn visual_clone(project: &ProjectEnvelope, id: &str) -> Clip {
    let mut clip = project.project.sequences[0].tracks[0].clips[0].clone();
    clip.id = ItemId::new(id).unwrap();
    clip.source = ClipSource::Generated {
        generator: Generator::Transparent,
    };
    clip.source_mapping = None;
    clip.audio = None;
    clip.effects.clear();
    clip.visual = Some(crate::test_support::identity_layout_visual());
    clip
}

fn visual_track(id: &str, order: i32, clips: Vec<Clip>) -> Track {
    Track {
        id: TrackId::new(id).unwrap(),
        kind: TrackKind::Visual,
        order,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips,
    }
}
