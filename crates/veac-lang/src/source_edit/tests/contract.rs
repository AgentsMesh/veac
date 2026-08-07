use super::*;

#[test]
fn contract_round_trips_and_has_a_schema() {
    let value = batch();
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(
        serde_json::from_str::<SourceEditBatch>(&json).unwrap(),
        value
    );

    let schema = schemars::schema_for!(SourceEditBatch);
    let encoded = serde_json::to_value(schema).unwrap();
    let encoded = encoded.to_string();
    assert!(encoded.contains("source_graph_sha256"));
    assert!(encoded.contains("set_expression"));
    assert!(encoded.contains("set_statement"));
    assert!(encoded.contains("statement_equals"));
    assert!(encoded.contains("body_statement"));
    assert!(encoded.contains("set_body"));
    assert!(encoded.contains("body_equals"));
    assert!(encoded.contains("set_declaration"));
    assert!(encoded.contains("declaration_equals"));
    assert!(encoded.contains("set_top_level_declaration"));
    assert!(encoded.contains("top_level_declaration_equals"));
    assert!(encoded.contains("insert_declaration"));
    assert!(encoded.contains("remove_declaration"));
    assert!(encoded.contains("insert_import"));
    assert!(encoded.contains("remove_import"));
    assert!(encoded.contains("import_equals"));
    assert!(encoded.contains(r#""minItems":1"#));
    assert!(encoded.contains(r#""maxItems":4096"#));
    assert!(encoded.contains(r#""maxLength":65536"#));
    assert!(encoded.contains("constant_value"));
    assert!(encoded.contains("body_expression"));
    assert!(encoded.contains("local_value"));
    assert!(encoded.contains(r#""maxItems":128"#));
    assert!(encoded.contains(r#""const":"https://veac.dev/schemas/source-edit""#));
    assert!(encoded.contains(r#""const":6"#));
    assert!(encoded.contains("temporal_animation"));
    assert!(encoded.contains("component_animation"));
    assert!(encoded.contains("function_body"));
    assert!(encoded.contains("method_body"));
    assert!(encoded.contains("enum_variant_field"));
    assert!(encoded.contains("struct_field_declaration"));
    assert!(encoded.contains("A-Za-z0-9"));
    for removed in ["preset", "resource", "modifier"] {
        assert!(!encoded.contains(removed), "schema retained {removed}");
    }
}

#[test]
fn contract_denies_unknown_fields() {
    let mut json = serde_json::to_value(batch()).unwrap();
    json.as_object_mut()
        .unwrap()
        .insert("surprise".to_owned(), serde_json::json!(true));
    assert!(serde_json::from_value::<SourceEditBatch>(json).is_err());

    let mut nested = serde_json::to_value(batch()).unwrap();
    nested["operations"][0]["target"]["path"]["surprise"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SourceEditBatch>(nested).is_err());
}

#[test]
fn target_json_carries_its_complete_typed_hierarchy() {
    let target = SourceNodeRef::item("main.veac", "showcase", "main", "overlays", "title");
    assert_eq!(
        serde_json::to_value(target).unwrap(),
        serde_json::json!({
            "module": "main.veac",
            "path": {
                "kind": "item",
                "project": "showcase",
                "sequence": "main",
                "layer": "overlays",
                "item": "title"
            }
        })
    );
}

#[test]
fn target_and_site_compatibility_is_closed() {
    assert!(ExpressionSite::ConstantValue.accepts(SourceNodeKind::Constant));
    assert!(!ExpressionSite::ConstantValue.accepts(SourceNodeKind::Item));
    let nested = ExpressionSite::BodyExpression {
        path: SourceExpressionPath::new(vec![SourceExpressionStep::BlockResult]),
    };
    assert!(nested.accepts(SourceNodeKind::Function));
    assert!(nested.accepts(SourceNodeKind::Method));
    assert!(!nested.accepts(SourceNodeKind::Constant));
    assert!(BodySite::TemporalAnimation {
        property: SourceTemporalProperty::VisualOpacity
    }
    .accepts(SourceNodeKind::Temporal));
    assert!(BodySite::ComponentAnimation {
        ordinal: 0,
        property: SourceTemporalProperty::VisualOpacity,
    }
    .accepts(SourceNodeKind::Function));
    assert!(DeclarationSite::ComponentAnimation { ordinal: 0 }.accepts(SourceNodeKind::Method));
    assert!(!DeclarationSite::ComponentAnimation { ordinal: 0 }.accepts(SourceNodeKind::Temporal));
    assert!(!BodySite::FunctionBody.accepts(SourceNodeKind::Item));
    let statement = StatementSite::BodyStatement {
        path: SourceExpressionPath::new(vec![SourceExpressionStep::LocalValue {
            operation: SourceLocalOperation::Let,
            binding: "value".to_owned(),
            ordinal: 0,
        }]),
    };
    assert!(statement.accepts(SourceNodeKind::Function));
    assert!(statement.accepts(SourceNodeKind::Method));
    assert!(!statement.accepts(SourceNodeKind::Constant));
    assert!(statement.is_valid());
}
