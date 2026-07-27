use super::*;
use crate::test_support::{multicam_project, time};

#[test]
fn multicam_group_insert_remove_round_trips_and_reports_changes() {
    let project = sample_project();
    let group = group("mcg_primary");
    let insert = batch(
        "op_insert_multicam",
        &project,
        vec![structure(StructureEdit::InsertMulticamGroup {
            group: Box::new(group.clone()),
            before_id: None,
            after_id: None,
        })],
    );
    let json = canonical_edit_batch_json(&insert).unwrap();
    assert_eq!(decode_edit_batch_json(&json).unwrap(), insert);
    assert!(json.contains("\"type\":\"insert_multicam_group\""));

    let (inserted, changed) = applied_with_changes(apply_edit_batch(&project, &insert));
    assert_eq!(
        inserted.project.multicam_groups.as_slice(),
        std::slice::from_ref(&group)
    );
    assert_changed(&changed, &group.id, &project.project.id);
    assert!(matches!(
        apply_edit_batch(&inserted, &insert),
        EditOutcome::NoChange {
            operation_recorded: true,
            ..
        }
    ));

    let mut reused = insert.clone();
    reused.operations = vec![structure(StructureEdit::RemoveMulticamGroup {
        group_id: group.id.clone(),
    })];
    assert!(matches!(
        apply_edit_batch(&inserted, &reused),
        EditOutcome::Conflict { diagnostics, .. }
            if diagnostics[0].code == "OPERATION_ID_REUSE"
    ));

    let remove = batch(
        "op_remove_multicam",
        &inserted,
        vec![structure(StructureEdit::RemoveMulticamGroup {
            group_id: group.id.clone(),
        })],
    );
    let (removed, changed) = applied_with_changes(apply_edit_batch(&inserted, &remove));
    assert!(removed.project.multicam_groups.is_empty());
    assert_changed(&changed, &group.id, &project.project.id);
    assert!(canonical_edit_batch_json(&remove)
        .unwrap()
        .contains("\"type\":\"remove_multicam_group\""));
    let schema = edit_batch_json_schema().unwrap().to_string();
    assert!(schema.contains("insert_multicam_group"));
    assert!(schema.contains("remove_multicam_group"));
}

#[test]
fn multicam_group_structure_rejects_collisions_references_locks_and_bad_values() {
    let project = multicam_project();
    let existing = project.project.multicam_groups[0].clone();
    let duplicate = batch(
        "op_duplicate_multicam",
        &project,
        vec![structure(StructureEdit::InsertMulticamGroup {
            group: Box::new(existing.clone()),
            before_id: None,
            after_id: None,
        })],
    );
    assert_message(
        apply_edit_batch(&project, &duplicate),
        "multicam group ID already exists",
    );

    let remove = |id: &str, value: &ProjectEnvelope| {
        batch(
            id,
            value,
            vec![structure(StructureEdit::RemoveMulticamGroup {
                group_id: existing.id.clone(),
            })],
        )
    };
    assert_message(
        apply_edit_batch(&project, &remove("op_used_multicam", &project)),
        "multicam group is still referenced",
    );
    let mut locked = project.clone();
    locked.project.sequences[0].tracks[0].state.locked = true;
    assert_message(
        apply_edit_batch(&locked, &remove("op_locked_multicam", &locked)),
        "multicam group is used by a clip on a locked track",
    );

    let invalid = batch(
        "op_invalid_multicam",
        &sample_project(),
        vec![structure(StructureEdit::InsertMulticamGroup {
            group: Box::new(MulticamGroup {
                angles: vec![existing.angles[0].clone()],
                ..group("mcg_invalid")
            }),
            before_id: None,
            after_id: None,
        })],
    );
    assert_rejected(
        apply_edit_batch(&sample_project(), &invalid),
        "MULTICAM_ANGLE_COUNT",
    );

    let atomic = batch(
        "op_atomic_multicam",
        &project,
        vec![
            structure(StructureEdit::InsertMulticamGroup {
                group: Box::new(group("mcg_archive")),
                before_id: Some(existing.id.clone()),
                after_id: None,
            }),
            structure(StructureEdit::RemoveMulticamGroup {
                group_id: existing.id.clone(),
            }),
        ],
    );
    assert_message(
        apply_edit_batch(&project, &atomic),
        "multicam group is still referenced",
    );
    assert_eq!(project.project.multicam_groups.len(), 1);
}

fn group(id: &str) -> MulticamGroup {
    MulticamGroup {
        id: MulticamGroupId::new(id).unwrap(),
        sync: MulticamSync {
            basis: MulticamSyncBasis::Manual,
            reference_angle_id: MulticamAngleId::new("ang_a").unwrap(),
        },
        angles: ["ang_a", "ang_b"]
            .into_iter()
            .map(|id| MulticamAngle {
                id: MulticamAngleId::new(id).unwrap(),
                material_id: MaterialId::new("med_video").unwrap(),
                source_offset: time(0),
            })
            .collect(),
    }
}

fn structure(edit: StructureEdit) -> EditOperation {
    EditOperation::EditStructure { edit }
}

fn applied_with_changes(outcome: EditOutcome) -> (ProjectEnvelope, Vec<ChangedObjectId>) {
    match outcome {
        EditOutcome::Applied {
            project,
            changed_objects,
            ..
        } => (project, changed_objects),
        other => panic!("expected applied, got {other:?}"),
    }
}

fn assert_changed(changed: &[ChangedObjectId], group: &MulticamGroupId, project: &ProjectId) {
    assert!(changed.contains(&ChangedObjectId::MulticamGroup { id: group.clone() }));
    assert!(changed.contains(&ChangedObjectId::Project {
        id: project.clone()
    }));
}

fn assert_message(outcome: EditOutcome, message: &str) {
    match outcome {
        EditOutcome::Rejected { diagnostics, .. } => {
            assert!(diagnostics.iter().any(|item| item.message == message));
        }
        other => panic!("expected rejected, got {other:?}"),
    }
}
