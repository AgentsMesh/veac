use super::*;

fn operation(source: &str) -> SourceEditOperation {
    SourceEditOperation::SetBody {
        target: SourceNodeRef::method("brand.veac", "Brand", "title"),
        site: BodySite::MethodBody,
        body: BodySource {
            source: source.into(),
        },
    }
}

#[test]
fn method_target_and_body_site_have_closed_json_shapes() {
    let target = SourceNodeRef::method("brand.veac", "Brand", "title");
    assert_eq!(
        serde_json::to_value(&target).unwrap(),
        serde_json::json!({
            "module": "brand.veac",
            "path": { "kind": "method", "receiver": "Brand", "method": "title" }
        })
    );
    assert_eq!(
        serde_json::to_value(BodySite::MethodBody).unwrap(),
        serde_json::json!({ "type": "method_body" })
    );
    assert_eq!(
        serde_json::from_value::<SourceNodeRef>(serde_json::to_value(target.clone()).unwrap())
            .unwrap(),
        target
    );
}

#[test]
fn method_body_accepts_only_method_targets_and_valid_blocks() {
    let target = SourceNodeRef::method("brand.veac", "Brand", "title");
    assert!(BodySite::MethodBody.accepts(SourceNodeKind::Method));
    assert!(BodySite::MethodBody.accepts_target(&target));
    assert!(!BodySite::MethodBody.accepts(SourceNodeKind::Function));
    assert!(!BodySite::FunctionBody.accepts_target(&target));

    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_method_body").unwrap(),
        revision('a'),
    );
    batch.operations.push(operation("{ self.title }"));
    assert_eq!(validate_source_edit_contract(&batch), Ok(()));
    batch.operations = vec![operation("self.title")];
    assert!(matches!(
        validate_source_edit_contract(&batch),
        Err(SourceEditError::InvalidBody(_))
    ));
}

#[test]
fn public_schema_describes_method_body_edits() {
    let schema = source_edit_batch_json_schema().unwrap().to_string();
    for token in ["method", "receiver", "method_body"] {
        assert!(schema.contains(token), "schema omitted {token}");
    }
}
