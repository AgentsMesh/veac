use super::*;

#[test]
fn definition_targets_and_sites_round_trip_as_closed_json() {
    let values = [
        (
            SourceNodeRef::component_local_instance("defs.veac", "card", "child"),
            ExpressionSite::ComponentLocalInstanceArgument {
                parameter: "duration".into(),
            },
        ),
        (
            SourceNodeRef::component_item("defs.veac", "card", "content", "title"),
            ExpressionSite::TextContent,
        ),
        (
            SourceNodeRef::preset("defs.veac", SourcePresetKind::TextStyle, "headline"),
            ExpressionSite::PresetTextStyleField {
                field: SourceTextStyleField::Size,
            },
        ),
        (
            SourceNodeRef::preset_audio_processor("defs.veac", "voice", "final-limiter"),
            ExpressionSite::PresetAudioProcessorField {
                processor_kind: SourceAudioProcessorKind::Limiter,
                field: SourceAudioProcessorField::Ceiling,
            },
        ),
        (
            SourceNodeRef::preset_audio_eq_band("defs.veac", "voice", "tone-shaper", "presence"),
            ExpressionSite::PresetAudioEqBandField {
                field: SourceAudioEqBandField::Gain,
            },
        ),
        (
            SourceNodeRef::preset_delivery_artifact(
                "defs.veac",
                "delivery",
                SourceDeliveryArtifactKind::AudioStem,
                "voice",
            ),
            ExpressionSite::PresetDeliveryField {
                field: SourceDeliveryField::SampleRate,
            },
        ),
    ];
    for (target, site) in values {
        let target_json = serde_json::to_string(&target).unwrap();
        let site_json = serde_json::to_string(&site).unwrap();
        assert_eq!(
            serde_json::from_str::<SourceNodeRef>(&target_json).unwrap(),
            target
        );
        assert_eq!(
            serde_json::from_str::<ExpressionSite>(&site_json).unwrap(),
            site
        );
    }
}

#[test]
fn definition_field_sites_reject_cross_kind_combinations() {
    for (target, site) in [
        (
            SourceNodeRef::preset("defs.veac", SourcePresetKind::TextStyle, "headline"),
            ExpressionSite::PresetColorField {
                field: SourceColorField::BasicExposure,
            },
        ),
        (
            SourceNodeRef::preset_audio_processor("defs.veac", "voice", "final-limiter"),
            ExpressionSite::PresetAudioProcessorField {
                processor_kind: SourceAudioProcessorKind::Limiter,
                field: SourceAudioProcessorField::Frequency,
            },
        ),
        (
            SourceNodeRef::preset_delivery_artifact(
                "defs.veac",
                "delivery",
                SourceDeliveryArtifactKind::Video,
                "master",
            ),
            ExpressionSite::PresetDeliveryField {
                field: SourceDeliveryField::SampleRate,
            },
        ),
    ] {
        assert!(!site.accepts_target(&target));
        let mut batch = SourceEditBatch::new(
            veac_ir::OperationId::new("op_definition_mismatch").unwrap(),
            revision('a'),
        );
        batch.operations.push(SourceEditOperation::SetExpression {
            target,
            site,
            expression: ExpressionSource { source: "1".into() },
        });
        assert_eq!(
            validate_source_edit_contract(&batch),
            Err(SourceEditError::IncompatibleExpressionSite)
        );
    }
}

#[test]
fn local_source_identity_is_canonical_and_never_hygienic() {
    let target =
        SourceNodeRef::component_modifier("defs.veac", "card", "content", "title", "sharpen");
    let json = serde_json::to_string(&target).unwrap();
    assert!(!json.contains('@'));
    assert!(!json.contains("veac-h-"));
    assert!(json.contains("component_modifier"));
}

#[test]
fn authored_definition_literals_reject_structural_injection() {
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_definition_injection").unwrap(),
        revision('a'),
    );
    batch.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::preset("defs.veac", SourcePresetKind::ColorPipeline, "grade"),
        site: ExpressionSite::PresetColorField {
            field: SourceColorField::BasicExposure,
        },
        expression: ExpressionSource {
            source: "1stops; output-space { range full; }".into(),
        },
    });
    assert!(matches!(
        validate_source_edit_contract(&batch),
        Err(SourceEditError::InvalidExpression(_))
    ));
}

#[test]
fn public_schemas_include_closed_definition_variants() {
    let schema = source_edit_batch_json_schema().unwrap().to_string();
    for value in [
        "component_local_instance",
        "preset_audio_processor",
        "preset_audio_eq_band",
        "preset_text_style_field",
        "delivery_profile",
        "channel_layout",
    ] {
        assert!(schema.contains(value), "schema omitted {value}");
    }
    assert!(!schema.contains("arbitrary_token"));
}
