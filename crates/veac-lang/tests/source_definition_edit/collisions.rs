use veac_lang::source_edit::{
    ExpressionSite, SourceAudioEqBandField, SourceAudioProcessorField, SourceAudioProcessorKind,
    SourceNodeRef, SourcePresetKind,
};

use super::fixture::{source_index, SOURCE};

#[test]
fn same_preset_name_across_kinds_and_child_kinds_never_collide() {
    let inventory = source_index().inventory();
    for target in [
        SourceNodeRef::preset("main.veac", SourcePresetKind::TextStyle, "shared"),
        SourceNodeRef::preset("main.veac", SourcePresetKind::TextLayout, "shared"),
        SourceNodeRef::preset(
            "main.veac",
            SourcePresetKind::ModifierStack,
            "shared-effects",
        ),
        SourceNodeRef::preset(
            "main.veac",
            SourcePresetKind::EffectPipeline,
            "shared-effects",
        ),
        SourceNodeRef::preset_modifier("main.veac", "shared-effects", "stacked-sharpen"),
        SourceNodeRef::preset_stage("main.veac", "shared-effects", "pipeline-blur"),
    ] {
        assert!(inventory.nodes.iter().any(|value| value.target == target));
    }
}

#[test]
fn same_kind_audio_processors_with_distinct_ids_are_addressable() {
    let duplicate = SOURCE.replace(
        "processor limiter final-limiter { ceiling -1db; attack 1ms; release 50ms; }",
        "processor limiter fast-limit { ceiling -1db; attack 1ms; release 50ms; }\n  processor limiter slow-limit { ceiling -2db; attack 2ms; release 60ms; }",
    );
    let inventory = veac_lang::program::compile_source(&duplicate)
        .unwrap()
        .source_index()
        .unwrap()
        .inventory();
    for (id, expected) in [("fast-limit", "-1db"), ("slow-limit", "-2db")] {
        let target = SourceNodeRef::preset_audio_processor("main.veac", "voice-chain", id);
        let node = inventory
            .nodes
            .iter()
            .find(|value| value.target == target)
            .unwrap();
        let site = ExpressionSite::PresetAudioProcessorField {
            processor_kind: SourceAudioProcessorKind::Limiter,
            field: SourceAudioProcessorField::Ceiling,
        };
        assert_eq!(
            node.expressions
                .iter()
                .find(|value| value.site == site)
                .unwrap()
                .source,
            expected
        );
    }
}

#[test]
fn repeated_singleton_color_sections_are_language_errors() {
    let original = "  basic {\n    exposure 0stops; temperature 6500k; tint 0;\n    highlights 0; shadows 0; fade 0%;\n  }";
    let duplicate = SOURCE.replace(original, &format!("{original}\n{original}"));
    let error = veac_lang::program::compile_source(&duplicate).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_PRESET_DEFINITION");
    assert!(error.as_slice()[0]
        .message
        .contains("AUTHORING_DUPLICATE_FIELD"));
}

#[test]
fn eq_processor_and_bands_have_explicit_hierarchical_identities() {
    let eq = SOURCE.replace(
        "processor limiter final-limiter { ceiling -1db; attack 1ms; release 50ms; }",
        "processor eq secondary-eq {\n    band low { frequency 100hz; gain 1db; q 0.7; }\n    band high { frequency 200hz; gain 2db; q 0.8; }\n  }",
    );
    let inventory = veac_lang::program::compile_source(&eq)
        .unwrap()
        .source_index()
        .unwrap()
        .inventory();
    let target = SourceNodeRef::preset_audio_processor("main.veac", "voice-chain", "secondary-eq");
    let node = inventory
        .nodes
        .iter()
        .find(|value| value.target == target)
        .unwrap();
    assert!(node.expressions.is_empty());
    for (band, expected) in [("low", "100hz"), ("high", "200hz")] {
        let target =
            SourceNodeRef::preset_audio_eq_band("main.veac", "voice-chain", "secondary-eq", band);
        let node = inventory
            .nodes
            .iter()
            .find(|value| value.target == target)
            .unwrap();
        let site = ExpressionSite::PresetAudioEqBandField {
            field: SourceAudioEqBandField::Frequency,
        };
        assert_eq!(
            node.expressions
                .iter()
                .find(|value| value.site == site)
                .unwrap()
                .source,
            expected
        );
    }
    let json = serde_json::to_string(&inventory).unwrap();
    assert!(json.contains("secondary-eq"));
    assert!(json.contains("\"band\":\"low\""));
    assert!(!json.contains("occurrence"));
    assert!(!json.contains("band_index"));
}

#[test]
fn duplicate_processor_and_eq_band_ids_are_language_errors() {
    for replacement in [
        "processor limiter duplicate { ceiling -1db; attack 1ms; release 50ms; }\n  processor limiter duplicate { ceiling -2db; attack 2ms; release 60ms; }",
        "processor eq duplicate-bands { band duplicate { frequency 100hz; gain 1db; q 0.7; } band duplicate { frequency 200hz; gain 2db; q 0.8; } }",
    ] {
        let source = SOURCE.replace(
            "processor limiter final-limiter { ceiling -1db; attack 1ms; release 50ms; }",
            replacement,
        );
        let error = veac_lang::program::compile_source(&source).unwrap_err();
        assert!(error.as_slice()[0].message.contains("AUTHORING_DUPLICATE_ID"));
    }
}

#[test]
fn component_local_names_are_scoped_by_their_owner_definition() {
    let second = r#"component sequence badge {
  body {
    layer visual @content {
      item @title {
        source generated transparent; record { at 0s; duration 1s; }
        modifiers { effect @sharpen { type video.sharpen; parameter amount 2; } }
      }
    }
  }
}
"#;
    let source = SOURCE.replacen(
        "component sequence child {",
        &format!("{second}component sequence child {{"),
        1,
    );
    let inventory = veac_lang::program::compile_source(&source)
        .unwrap()
        .source_index()
        .unwrap()
        .inventory();
    for component in ["badge", "card"] {
        let target = SourceNodeRef::component_modifier(
            "main.veac",
            component,
            "content",
            "title",
            "sharpen",
        );
        assert!(inventory.nodes.iter().any(|value| value.target == target));
    }
}
