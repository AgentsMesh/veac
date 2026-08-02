use super::*;

#[test]
fn insert_move_set_and_remove_apply_preserve_order_and_changes() {
    let project = sample_project();
    let first = apply(
        "apl_first",
        "aps_first",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    let second = apply(
        "apl_second",
        "aps_second",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    let insert = batch(
        "op_apply_insert",
        &project,
        vec![
            structure(StructureEdit::InsertApply {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                apply: Box::new(first.clone()),
                before_id: None,
                after_id: None,
            }),
            structure(StructureEdit::InsertApply {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                apply: Box::new(second),
                before_id: Some(first.id.clone()),
                after_id: None,
            }),
        ],
    );
    let EditOutcome::Applied {
        project: inserted,
        changed_objects,
        ..
    } = apply_edit_batch(&project, &insert)
    else {
        panic!("insert rejected");
    };
    assert_eq!(apply_order(&inserted), vec!["apl_second", "apl_first"]);
    assert!(changed_objects.contains(&ChangedObjectId::Apply {
        id: first.id.clone()
    }));

    let moved = applied(apply_edit_batch(
        &inserted,
        &batch(
            "op_apply_move",
            &inserted,
            vec![structure(StructureEdit::MoveApply {
                apply_id: ApplyId::new("apl_second").unwrap(),
                before_id: None,
                after_id: None,
            })],
        ),
    ));
    assert_eq!(apply_order(&moved), vec!["apl_first", "apl_second"]);

    let mut replacement = moved.project.sequences[0].applies[0].clone();
    replacement.enabled = false;
    let set = applied(apply_edit_batch(
        &moved,
        &batch(
            "op_apply_set",
            &moved,
            vec![structure(StructureEdit::SetApply {
                apply_id: replacement.id.clone(),
                apply: Box::new(replacement),
            })],
        ),
    ));
    assert!(!set.project.sequences[0].applies[0].enabled);

    let removed = applied(apply_edit_batch(
        &set,
        &batch(
            "op_apply_remove",
            &set,
            vec![structure(StructureEdit::RemoveApply {
                apply_id: ApplyId::new("apl_first").unwrap(),
            })],
        ),
    ));
    assert_eq!(apply_order(&removed), vec!["apl_second"]);
}

#[test]
fn remove_apply_cascades_its_matte_and_marks_top_level_changes() {
    let mut project = sample_project();
    let mut apply = apply(
        "apl_matte",
        "aps_matte",
        ApplyTarget::Layer {
            track_id: TrackId::new("trk_video").unwrap(),
        },
    );
    apply.record_range = project.project.sequences[0].tracks[1].clips[0].record_range;
    project.project.sequences[0].applies.push(apply.clone());
    let relation = matte(&apply.id);
    project.project.relations.push(relation.clone());
    let outcome = apply_edit_batch(
        &project,
        &batch(
            "op_apply_matte_remove",
            &project,
            vec![structure(StructureEdit::RemoveApply {
                apply_id: apply.id.clone(),
            })],
        ),
    );
    let EditOutcome::Applied {
        project: removed,
        changed_objects,
        ..
    } = outcome
    else {
        panic!("remove rejected");
    };
    assert!(removed.project.sequences[0].applies.is_empty());
    assert!(removed.project.relations.is_empty());
    assert!(changed_objects.contains(&ChangedObjectId::Relation { id: relation.id }));
    assert!(changed_objects.contains(&ChangedObjectId::Project {
        id: removed.project.id.clone()
    }));
}

pub(super) fn apply(id: &str, stage: &str, target: ApplyTarget) -> Apply {
    Apply {
        id: ApplyId::new(id).unwrap(),
        enabled: true,
        record_range: range(0, 600),
        target,
        stages: vec![ApplyStage {
            id: ApplyStageId::new(stage).unwrap(),
            enabled: true,
            active_range: None,
            operation: ApplyOperation::Color {
                pipeline: empty_pipeline(),
            },
        }],
        mix: ApplyMix::default(),
    }
}

pub(super) fn matte(apply_id: &ApplyId) -> Relation {
    Relation {
        id: RelationId::new("rel_apply_edit").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new("itm_caption").unwrap()),
            consumer: RelationEndpoint::apply(apply_id.clone()),
            parameters: MatteRelationParameters {
                mode: TrackMatteMode::Alpha,
                invert: false,
            },
        },
    }
}

fn empty_pipeline() -> ColorPipeline {
    let space = ColorSpace {
        primaries: ColorPrimaries::Bt709,
        transfer: ColorTransfer::Bt709,
        matrix: ColorMatrix::Bt709,
        range: ColorRange::Limited,
    };
    ColorPipeline {
        input: space,
        working: space,
        output: space,
        stages: vec![],
    }
}

fn structure(edit: StructureEdit) -> EditOperation {
    EditOperation::EditStructure { edit }
}

fn apply_order(project: &ProjectEnvelope) -> Vec<&str> {
    project.project.sequences[0]
        .applies
        .iter()
        .map(|apply| apply.id.as_str())
        .collect()
}
