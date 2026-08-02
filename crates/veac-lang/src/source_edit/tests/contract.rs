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
    assert!(encoded.contains(r#""minItems":1"#));
    assert!(encoded.contains(r#""maxItems":4096"#));
    assert!(encoded.contains(r#""maxLength":65536"#));
    assert!(encoded.contains("item_enabled"));
    assert!(encoded.contains("modifier"));
    assert!(encoded.contains("stage"));
    assert!(encoded.contains("sequence"));
    assert!(encoded.contains("A-Za-z0-9"));
    assert!(!encoded.contains("component_instance_override"));
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
    let target =
        SourceNodeRef::modifier("main.veac", "showcase", "main", "overlays", "title", "blur");
    assert_eq!(
        serde_json::to_value(target).unwrap(),
        serde_json::json!({
            "module": "main.veac",
            "path": {
                "kind": "modifier",
                "project": "showcase",
                "sequence": "main",
                "layer": "overlays",
                "item": "title",
                "modifier": "blur"
            }
        })
    );
}

#[test]
fn target_and_site_compatibility_is_closed() {
    assert!(ExpressionSite::ItemEnabled.accepts(SourceNodeKind::Item));
    assert!(!ExpressionSite::ItemEnabled.accepts(SourceNodeKind::Sequence));
    assert!(ExpressionSite::ComponentInstanceArgument {
        parameter: "title".to_owned()
    }
    .accepts(SourceNodeKind::ComponentInstance));
    assert!(ExpressionSite::ModifierParameter {
        parameter: "amount".to_owned()
    }
    .accepts(SourceNodeKind::Stage));

    let named = [
        ExpressionSite::ComponentParameterDefault {
            parameter: "title".to_owned(),
        },
        ExpressionSite::ComponentInstanceArgument {
            parameter: "title".to_owned(),
        },
        ExpressionSite::ModifierParameter {
            parameter: "amount".to_owned(),
        },
    ];
    assert!(named.iter().all(|value| value.name().is_some()));
    assert_eq!(ExpressionSite::TextContent.name(), None);
}
