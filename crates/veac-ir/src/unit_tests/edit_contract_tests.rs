use std::error::Error;

use crate::*;

fn batch() -> EditBatch {
    EditBatch {
        operation_id: OperationId::new("op_contract").unwrap(),
        base_revision: 0,
        atomic: true,
        preconditions: vec![],
        operations: vec![EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_video").unwrap(),
            enabled: false,
        }],
    }
}

#[test]
fn edit_batch_json_is_strict_canonical_and_schema_discoverable() {
    let value = batch();
    let json = canonical_edit_batch_json(&value).unwrap();
    assert_eq!(decode_edit_batch_json(&json).unwrap(), value);
    assert_eq!(canonical_edit_batch_json(&value).unwrap(), json);
    assert_eq!(edit_batch_json_schema().unwrap()["title"], "EditBatch");
    assert_eq!(edit_outcome_json_schema().unwrap()["title"], "EditOutcome");

    let duplicate = json.replacen("\"atomic\":true", "\"atomic\":true,\"atomic\":true", 1);
    let error = decode_edit_batch_json(&duplicate).unwrap_err();
    assert!(error.to_string().contains("duplicate"));
    assert!(error.source().is_some());
}

#[test]
fn malformed_batch_contracts_fail_before_application() {
    let mut cases = Vec::new();
    let mut value = batch();
    value.atomic = false;
    cases.push(value);
    let mut value = batch();
    value.operations.clear();
    cases.push(value);
    let mut value = batch();
    value.base_revision = MAX_SAFE_INTEGER + 1;
    cases.push(value);
    for value in cases {
        let error = canonical_edit_batch_json(&value).unwrap_err();
        assert!(matches!(error, EditJsonError::Invalid(_)));
        assert!(error.source().is_none());
    }
    assert!(decode_edit_batch_json("not-json").is_err());
}

#[test]
fn edit_outcomes_have_deterministic_canonical_json() {
    let project = crate::test_support::sample_project();
    let outcome = apply_edit_batch(&project, &batch());
    let first = canonical_edit_outcome_json(&outcome).unwrap();
    let second = canonical_edit_outcome_json(&outcome).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        serde_json::from_str::<EditOutcome>(&first).unwrap(),
        outcome
    );
}
