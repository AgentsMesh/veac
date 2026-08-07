use super::*;

fn function_operation(name: &str, source: &str) -> SourceEditOperation {
    SourceEditOperation::SetBody {
        target: SourceNodeRef::function("motion.veac", name),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: source.to_owned(),
        },
    }
}

#[test]
fn function_target_and_body_site_have_closed_json_shapes() {
    let target = SourceNodeRef::function("motion.veac", "breathe");
    assert_eq!(
        serde_json::to_value(&target).unwrap(),
        serde_json::json!({
            "module": "motion.veac",
            "path": { "kind": "function", "function": "breathe" }
        })
    );
    assert_eq!(
        serde_json::to_value(BodySite::FunctionBody).unwrap(),
        serde_json::json!({ "type": "function_body" })
    );
    assert_eq!(
        serde_json::from_value::<SourceNodeRef>(serde_json::to_value(target.clone()).unwrap())
            .unwrap(),
        target
    );
}

#[test]
fn function_body_only_accepts_function_targets() {
    assert!(BodySite::FunctionBody.accepts(SourceNodeKind::Function));
    assert!(
        BodySite::FunctionBody.accepts_target(&SourceNodeRef::function("motion.veac", "breathe"))
    );
    assert!(!BodySite::FunctionBody.accepts(SourceNodeKind::Constant));
    assert!(!BodySite::FunctionBody.accepts_target(&SourceNodeRef::constant("motion.veac", "tau")));
}

#[test]
fn function_body_requires_a_parsed_root_block_and_valid_target() {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_function_body").unwrap(),
        revision('a'),
    );
    batch
        .operations
        .push(function_operation("breathe", "{ 1.0 + progress * 0.04 }"));
    assert_eq!(validate_source_edit_contract(&batch), Ok(()));

    batch.operations = vec![function_operation("bad.name", "{ progress }")];
    assert!(matches!(
        validate_source_edit_contract(&batch),
        Err(SourceEditError::InvalidNodeId(name)) if name == "bad.name"
    ));

    for source in ["progress", "{ progress + }"] {
        batch.operations = vec![function_operation("breathe", source)];
        assert!(matches!(
            validate_source_edit_contract(&batch),
            Err(SourceEditError::InvalidBody(_))
        ));
    }

    batch.operations = vec![SourceEditOperation::SetBody {
        target: SourceNodeRef::constant("motion.veac", "tau"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 1 }".into(),
        },
    }];
    assert_eq!(
        validate_source_edit_contract(&batch),
        Err(SourceEditError::IncompatibleBodySite)
    );
}

#[test]
fn expression_operation_cannot_encode_a_function_body_site() {
    let invalid = serde_json::json!({
        "type": "set_expression",
        "target": serde_json::to_value(SourceNodeRef::function("motion.veac", "breathe")).unwrap(),
        "site": { "type": "function_body" },
        "expression": { "source": "progress" }
    });
    assert!(serde_json::from_value::<SourceEditOperation>(invalid).is_err());
}

#[test]
fn public_schema_describes_function_edits() {
    let schema = source_edit_batch_json_schema().unwrap().to_string();
    for token in ["function", "function_body", "set_body", "body_equals"] {
        assert!(schema.contains(token), "schema omitted {token}");
    }
}
