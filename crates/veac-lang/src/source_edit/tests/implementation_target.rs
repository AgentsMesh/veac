use super::*;

fn implementation(identity: &str) -> SourceNodeRef {
    SourceNodeRef::implementation("brand.veac", "Card", identity)
}

fn implementation_batch(identity: &str) -> SourceEditBatch {
    let mut value = batch();
    value.operations = vec![SourceEditOperation::RemoveDeclaration {
        target: implementation(identity),
    }];
    value
}

#[test]
fn implementation_target_publishes_receiver_and_stable_identity() {
    let target = implementation("presentation");
    assert_eq!(target.path.identifiers(), vec!["Card", "presentation"]);
    validate_source_edit_contract(&implementation_batch("presentation")).unwrap();
}

#[test]
fn implementation_target_requires_its_identity_in_strict_json() {
    let mut value = serde_json::to_value(implementation_batch("presentation")).unwrap();
    value["operations"][0]["target"]["path"]
        .as_object_mut()
        .unwrap()
        .remove("implementation");
    assert!(matches!(
        decode_source_edit_batch_json(&serde_json::to_string(&value).unwrap()),
        Err(SourceEditJsonError::Json(_))
    ));

    let path = value["operations"][0]["target"]["path"]
        .as_object_mut()
        .unwrap();
    path.insert("implementation".into(), serde_json::json!("presentation"));
    path.insert("ordinal".into(), serde_json::json!(0));
    assert!(matches!(
        decode_source_edit_batch_json(&serde_json::to_string(&value).unwrap()),
        Err(SourceEditJsonError::Json(_))
    ));
}

#[test]
fn invalid_implementation_identity_fails_contract_validation() {
    assert!(matches!(
        validate_source_edit_contract(&implementation_batch("bad.identity")),
        Err(SourceEditError::InvalidNodeId(value)) if value == "bad.identity"
    ));
}
