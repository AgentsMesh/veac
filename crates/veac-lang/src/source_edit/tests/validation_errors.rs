use super::*;

struct EmptySnapshot;

impl SourceSnapshot for EmptySnapshot {
    fn node_exists(&self, _: &SourceNodeRef) -> bool {
        false
    }

    fn expression_source(&self, _: &SourceNodeRef, _: &ExpressionSite) -> Option<&str> {
        None
    }

    fn statement_source(&self, _: &SourceNodeRef, _: &StatementSite) -> Option<&str> {
        None
    }

    fn body_source(&self, _: &SourceNodeRef, _: BodySite) -> Option<&str> {
        None
    }

    fn declaration_source(&self, _: &SourceNodeRef, _: DeclarationSite) -> Option<&str> {
        None
    }
}

#[test]
fn contract_rejects_version_operation_id_and_invalid_node_ids() {
    let mut value = batch();
    value.schema_version = 1;
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::UnsupportedSchemaVersion(1))
    );

    let mut json = serde_json::to_value(batch()).unwrap();
    json["operation_id"] = serde_json::json!("invalid");
    let invalid_id: SourceEditBatch = serde_json::from_value(json).unwrap();
    assert!(matches!(
        validate_source_edit_contract(&invalid_id),
        Err(SourceEditError::InvalidOperationId(_))
    ));

    value = batch();
    value.operations = vec![SourceEditOperation::SetExpression {
        target: SourceNodeRef::constant("timeline/main.veac", "bad.id"),
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: "true".to_owned(),
        },
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidNodeId(_))
    ));

    value.operations = vec![SourceEditOperation::SetBody {
        target: SourceNodeRef::item(
            "timeline/main.veac",
            "timeline",
            "bad.sequence",
            "visual",
            "hero",
        ),
        site: BodySite::TemporalAnimation {
            property: SourceTemporalProperty::VisualOpacity,
        },
        body: BodySource {
            source: "{ 1 }".to_owned(),
        },
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidNodeId(id)) if id == "bad.sequence"
    ));
}

#[test]
fn source_targets_use_the_canonical_language_name_contract() {
    for valid in ["_private", "title-card", "name_2"] {
        let mut value = batch();
        value.operations = vec![SourceEditOperation::SetExpression {
            target: SourceNodeRef::constant("main.veac", valid),
            site: ExpressionSite::ConstantValue,
            expression: ExpressionSource {
                source: "true".to_owned(),
            },
        }];
        assert_eq!(validate_source_edit_contract(&value), Ok(()), "{valid}");
    }
    for invalid in [
        "2fast",
        ".hidden",
        "trailing-",
        "two--dash",
        "标题",
        "true",
        "false",
    ] {
        let mut value = batch();
        value.operations = vec![SourceEditOperation::SetExpression {
            target: SourceNodeRef::constant("main.veac", invalid),
            site: ExpressionSite::ConstantValue,
            expression: ExpressionSource {
                source: "true".to_owned(),
            },
        }];
        assert!(matches!(
            validate_source_edit_contract(&value),
            Err(SourceEditError::InvalidNodeId(id)) if id == invalid
        ));
    }
}

#[test]
fn contract_rejects_empty_large_nul_and_invalid_current_revision() {
    for source in [String::new(), "x".repeat(65_537), "\0".to_owned()] {
        let mut value = batch();
        value.operations = vec![operation(&source)];
        assert!(matches!(
            validate_source_edit_contract(&value),
            Err(SourceEditError::InvalidExpression(_))
        ));
    }
    let current = SourceRevision {
        source_graph_sha256: "BAD".to_owned(),
    };
    assert!(matches!(
        validate_source_edit_batch(&batch(), &current, &EmptySnapshot),
        Err(SourceEditError::InvalidDigest(_))
    ));
}

#[test]
fn contract_requires_exactly_one_executable_expression() {
    for source in [
        "2s; const text injected = \"x\"",
        "${duration}; item injected {}",
        "duration trailing",
    ] {
        let mut value = batch();
        value.operations = vec![operation(source)];
        assert!(matches!(
            validate_source_edit_contract(&value),
            Err(SourceEditError::InvalidExpression(_))
        ));
    }
    let mut wrapped = batch();
    wrapped.operations = vec![operation("${duration}")];
    assert_eq!(validate_source_edit_contract(&wrapped), Ok(()));
}

#[test]
fn contract_enforces_precondition_and_operation_budgets() {
    let mut value = batch();
    value.preconditions = vec![
        SourcePrecondition::NodeAbsent { target: target() };
        MAX_SOURCE_EDIT_PRECONDITIONS + 1
    ];
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::TooManyPreconditions {
            limit: MAX_SOURCE_EDIT_PRECONDITIONS
        })
    );

    value.preconditions.clear();
    value.operations = vec![operation("1s"); MAX_SOURCE_EDIT_OPERATIONS + 1];
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::TooManyOperations {
            limit: MAX_SOURCE_EDIT_OPERATIONS
        })
    );
}
