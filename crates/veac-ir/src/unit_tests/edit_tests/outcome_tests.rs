use super::*;

#[test]
fn batch_applies_multiple_changes_atomically_and_records_id() {
    let project = sample_project();
    let edit = batch(
        "op_text",
        &project,
        vec![
            EditOperation::SetText {
                clip_id: ItemId::new("itm_caption").unwrap(),
                text: "updated".to_owned(),
            },
            EditOperation::SetClipEnabled {
                clip_id: ItemId::new("itm_caption").unwrap(),
                enabled: false,
            },
        ],
    );
    match apply_edit_batch(&project, &edit) {
        EditOutcome::Applied {
            project,
            new_revision,
            changed_objects,
            normalized_operations,
        } => {
            assert_eq!(new_revision, 8);
            assert_eq!(
                changed_objects,
                vec![ChangedObjectId::Item {
                    id: ItemId::new("itm_caption").unwrap()
                }]
            );
            assert_eq!(normalized_operations, edit.operations);
            assert_eq!(project.project.applied_operations[0].id, edit.operation_id);
            assert_eq!(project.project.applied_operations[0].request_hash.len(), 64);
            let caption = &project.project.sequences[0].tracks[1].clips[0];
            assert!(!caption.enabled);
            assert!(
                matches!(&caption.source, ClipSource::Caption { text, .. } if text == "updated")
            );
        }
        other => panic!("expected applied, got {other:?}"),
    }
}

#[test]
fn accepted_no_change_retry_and_id_reuse_are_distinct() {
    let project = sample_project();
    let edit = batch(
        "op_no_change",
        &project,
        vec![EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_caption").unwrap(),
            enabled: true,
        }],
    );
    let recorded = match apply_edit_batch(&project, &edit) {
        EditOutcome::NoChange {
            project,
            current_revision: 8,
            operation_recorded: true,
        } => project,
        other => panic!("expected no change, got {other:?}"),
    };
    assert!(matches!(
        apply_edit_batch(&recorded, &edit),
        EditOutcome::NoChange {
            current_revision: 8,
            operation_recorded: true,
            ..
        }
    ));
    let mut reused = edit;
    reused.operations = vec![EditOperation::SetClipEnabled {
        clip_id: ItemId::new("itm_caption").unwrap(),
        enabled: false,
    }];
    assert!(matches!(
        apply_edit_batch(&recorded, &reused),
        EditOutcome::Conflict { diagnostics, .. } if diagnostics[0].code == "OPERATION_ID_REUSE"
    ));
}

#[test]
fn stale_and_malformed_batches_fail_before_mutation() {
    let project = sample_project();
    let operation = EditOperation::SetClipEnabled {
        clip_id: ItemId::new("itm_caption").unwrap(),
        enabled: false,
    };
    let mut stale = batch("op_stale", &project, vec![operation.clone()]);
    stale.base_revision = 6;
    assert!(matches!(
        apply_edit_batch(&project, &stale),
        EditOutcome::Conflict { current_revision: 7, diagnostics }
            if diagnostics[0].code == "STALE_REVISION"
    ));

    let mut malformed = batch("op_empty", &project, vec![]);
    assert_rejected(apply_edit_batch(&project, &malformed), "INVALID_EDIT_BATCH");
    malformed.atomic = false;
    assert_rejected(apply_edit_batch(&project, &malformed), "INVALID_EDIT_BATCH");
    malformed.operation_id = serde_json::from_str("\"wrong\"").unwrap();
    assert_rejected(apply_edit_batch(&project, &malformed), "INVALID_EDIT_BATCH");

    let mut invalid = project.clone();
    invalid.project.timebase = 0;
    let edit = batch("op_invalid_base", &invalid, vec![operation]);
    assert_rejected(apply_edit_batch(&invalid, &edit), "TIMEBASE");
}

#[test]
fn outcomes_are_serializable_protocol_values() {
    let project = sample_project();
    let edit = batch(
        "op_protocol",
        &project,
        vec![EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_caption").unwrap(),
            enabled: false,
        }],
    );
    let outcome = apply_edit_batch(&project, &edit);
    let json = serde_json::to_string(&outcome).unwrap();
    assert!(json.contains("\"status\":\"applied\""));
    assert_eq!(serde_json::from_str::<EditOutcome>(&json).unwrap(), outcome);
}
