use veac_ir::{
    apply_edit_batch, canonical_json, decode_canonical_json, project_json_schema, ChangedObjectId,
    EditBatch, EditOperation, EditOutcome, ItemId, OperationId,
};

const FIXTURE: &str = include_str!("fixtures/minimal-project.json");

#[test]
fn public_json_and_edit_contract_round_trip() {
    let project = decode_canonical_json(FIXTURE).unwrap();
    let canonical = canonical_json(&project).unwrap();
    assert_eq!(decode_canonical_json(&canonical).unwrap(), project);
    assert!(!canonical.contains('\n'));

    let batch = EditBatch {
        operation_id: OperationId::new("op_disable_video").unwrap(),
        base_revision: 0,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_video").unwrap(),
            enabled: false,
        }],
    };
    let edited = match apply_edit_batch(&project, &batch) {
        EditOutcome::Applied {
            project,
            new_revision,
            changed_objects,
            ..
        } => {
            assert_eq!(new_revision, 1);
            assert_eq!(
                changed_objects,
                vec![ChangedObjectId::Item {
                    id: ItemId::new("itm_video").unwrap()
                }]
            );
            project
        }
        outcome => panic!("unexpected edit result: {outcome:?}"),
    };
    assert!(canonical_json(&edited)
        .unwrap()
        .contains("op_disable_video"));
}

#[test]
fn public_schema_and_strict_decoder_are_available_to_tools() {
    let schema = project_json_schema().unwrap();
    assert!(schema.get("$schema").is_some());
    let schema_text = serde_json::to_string(&schema).unwrap();
    assert!(schema_text.contains("^prj_"));
    assert!(schema_text.contains("9007199254740991"));

    let with_unknown = FIXTURE.replacen(
        "\"schema_version\": 3,",
        "\"schema_version\": 3, \"unknown\": true,",
        1,
    );
    assert!(decode_canonical_json(&with_unknown).is_err());
}
