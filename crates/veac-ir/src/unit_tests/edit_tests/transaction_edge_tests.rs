use super::*;

#[test]
fn batch_contract_rejects_unsafe_revision_and_non_ijson_values() {
    let project = sample_project();
    let operation = EditOperation::SetClipEnabled {
        clip_id: ItemId::new("itm_caption").unwrap(),
        enabled: false,
    };
    let mut unsafe_revision = batch("op_unsafe_revision", &project, vec![operation]);
    unsafe_revision.base_revision = MAX_SAFE_INTEGER + 1;
    assert_invalid_batch(&project, &unsafe_revision);

    let non_ijson = batch(
        "op_non_ijson",
        &project,
        vec![EditOperation::EditEffectParameter {
            edit: EffectParameterEdit::Set {
                clip_id: ItemId::new("itm_video").unwrap(),
                effect_id: EffectId::new("fx_color").unwrap(),
                name: "brightness".to_owned(),
                value: ParameterValue::Time {
                    value: RationalTime {
                        value: i64::MAX,
                        timescale: 600,
                    },
                },
            },
        }],
    );
    assert_invalid_batch(&project, &non_ijson);
}

#[test]
fn maximum_canonical_revision_fails_before_recording_an_operation() {
    let mut project = sample_project();
    project.project.revision = MAX_SAFE_INTEGER;
    validate(&project).unwrap();
    let edit = batch(
        "op_revision_limit",
        &project,
        vec![EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_caption").unwrap(),
            enabled: true,
        }],
    );
    match apply_edit_batch(&project, &edit) {
        EditOutcome::Rejected {
            current_revision,
            diagnostics,
        } => {
            assert_eq!(current_revision, MAX_SAFE_INTEGER);
            assert_eq!(diagnostics[0].code, "REVISION_OVERFLOW");
            assert_eq!(diagnostics[0].pointer, "/project/revision");
        }
        other => panic!("expected revision rejection, got {other:?}"),
    }
    assert!(project.project.applied_operations.is_empty());
}

fn assert_invalid_batch(project: &ProjectEnvelope, batch: &EditBatch) {
    match apply_edit_batch(project, batch) {
        EditOutcome::Rejected {
            current_revision,
            diagnostics,
        } => {
            assert_eq!(current_revision, project.project.revision);
            assert_eq!(diagnostics[0].code, "INVALID_EDIT_BATCH");
            assert_eq!(diagnostics[0].pointer, "/");
        }
        other => panic!("expected invalid batch, got {other:?}"),
    }
}
