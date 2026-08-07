use super::*;

#[test]
fn component_animation_sites_have_stable_typed_json() {
    assert_eq!(
        serde_json::to_value(BodySite::ComponentAnimation {
            ordinal: 3,
            property: SourceTemporalProperty::VisualOpacity,
        })
        .unwrap(),
        serde_json::json!({
            "type": "component_animation",
            "ordinal": 3,
            "property": "visual_opacity"
        })
    );
    assert_eq!(
        serde_json::to_value(DeclarationSite::ComponentAnimation { ordinal: 3 }).unwrap(),
        serde_json::json!({"type": "component_animation", "ordinal": 3})
    );
}

#[test]
fn component_animation_body_and_declaration_fragments_are_distinct() {
    let function = SourceNodeRef::function("main.veac", "animated");
    let mut body = batch();
    body.operations = vec![SourceEditOperation::SetBody {
        target: function.clone(),
        site: BodySite::ComponentAnimation {
            ordinal: 0,
            property: SourceTemporalProperty::VisualOpacity,
        },
        body: BodySource {
            source: "{ clamp(progress, 0.0, 1.0) }".into(),
        },
    }];
    assert_eq!(validate_source_edit_contract(&body), Ok(()));

    let mut declaration = batch();
    declaration.operations = vec![SourceEditOperation::SetDeclaration {
        target: function,
        site: DeclarationSite::ComponentAnimation { ordinal: 0 },
        declaration: DeclarationSource {
            source: "animate visual-opacity on clip(card) { progress }".into(),
        },
    }];
    assert_eq!(validate_source_edit_contract(&declaration), Ok(()));
}

#[test]
fn component_animation_edits_reject_wrong_fragment_or_target_kind() {
    let function = SourceNodeRef::function("main.veac", "animated");
    for source in ["{ progress }", "progress"] {
        let mut value = batch();
        value.operations = vec![SourceEditOperation::SetDeclaration {
            target: function.clone(),
            site: DeclarationSite::ComponentAnimation { ordinal: 0 },
            declaration: DeclarationSource {
                source: source.into(),
            },
        }];
        assert!(matches!(
            validate_source_edit_contract(&value),
            Err(SourceEditError::InvalidDeclaration(_))
        ));
    }

    let mut incompatible = batch();
    incompatible.operations = vec![SourceEditOperation::SetBody {
        target: SourceNodeRef::constant("main.veac", "value"),
        site: BodySite::ComponentAnimation {
            ordinal: 0,
            property: SourceTemporalProperty::VisualOpacity,
        },
        body: BodySource {
            source: "{ progress }".into(),
        },
    }];
    assert_eq!(
        validate_source_edit_contract(&incompatible),
        Err(SourceEditError::IncompatibleBodySite)
    );
}
