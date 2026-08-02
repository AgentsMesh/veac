use veac_lang::source_edit::{
    ExpressionSite, SourceAudioEqBandField, SourceAudioProcessorField, SourceAudioProcessorKind,
    SourceColorField, SourceDeliveryArtifactKind, SourceDeliveryField, SourceNodeRef,
    SourcePresetKind, SourceTextLayoutField, SourceTextStyleField,
};

use super::fixture::{source_index, SOURCE};

#[path = "inventory/support.rs"]
mod support;
use support::*;

#[test]
fn component_definition_targets_are_owner_hierarchical_and_source_exact() {
    let inventory = source_index().inventory();
    let targets = [
        SourceNodeRef::component_local_instance("main.veac", "card", "child"),
        SourceNodeRef::component_layer("main.veac", "card", "content"),
        SourceNodeRef::component_item("main.veac", "card", "content", "title"),
        SourceNodeRef::component_modifier("main.veac", "card", "content", "title", "sharpen"),
        SourceNodeRef::component_apply("main.veac", "card", "finish"),
        SourceNodeRef::component_stage("main.veac", "card", "finish", "polish"),
    ];
    for target in targets {
        let node = inventory
            .nodes
            .iter()
            .find(|value| value.target == target)
            .unwrap();
        assert!(SOURCE[node.range.start..node.range.end].starts_with('@'));
    }
    assert_source(
        &inventory,
        SourceNodeRef::component_item("main.veac", "card", "content", "title"),
        ExpressionSite::TextContent,
        "\"before\"",
    );
    for (site, expected) in [
        (ExpressionSite::ItemRecordStart, "0s"),
        (ExpressionSite::ItemRecordDuration, "1s"),
        (ExpressionSite::ItemEnabled, "enabled"),
    ] {
        assert_source(
            &inventory,
            SourceNodeRef::component_item("main.veac", "card", "content", "title"),
            site,
            expected,
        );
    }
    assert_source(
        &inventory,
        SourceNodeRef::component_modifier("main.veac", "card", "content", "title", "sharpen"),
        parameter("amount"),
        "1",
    );
    assert_source(
        &inventory,
        SourceNodeRef::component_local_instance("main.veac", "card", "child"),
        ExpressionSite::ComponentLocalInstanceArgument {
            parameter: "duration".into(),
        },
        "500ms",
    );
}

#[test]
fn every_typed_preset_definition_exposes_closed_fields() {
    let inventory = source_index().inventory();
    for (kind, name) in [
        (SourcePresetKind::TextStyle, "shared"),
        (SourcePresetKind::TextLayout, "shared"),
        (SourcePresetKind::ModifierStack, "shared-effects"),
        (SourcePresetKind::EffectPipeline, "shared-effects"),
        (SourcePresetKind::ColorPipeline, "finishing-color"),
        (SourcePresetKind::AudioProcessors, "voice-chain"),
        (SourcePresetKind::DeliveryProfile, "wave-delivery"),
    ] {
        let target = SourceNodeRef::preset("main.veac", kind, name);
        assert!(inventory.nodes.iter().any(|value| value.target == target));
    }
    let cases = [
        preset(
            SourcePresetKind::TextStyle,
            "shared",
            style(SourceTextStyleField::Size),
            "24px",
        ),
        preset(
            SourcePresetKind::TextLayout,
            "shared",
            layout(SourceTextLayoutField::BoxWidth),
            "320px",
        ),
        node(
            SourceNodeRef::preset_modifier("main.veac", "shared-effects", "stacked-sharpen"),
            parameter("amount"),
            "1.25",
        ),
        node(
            SourceNodeRef::preset_stage("main.veac", "shared-effects", "pipeline-blur"),
            parameter("radius"),
            "2",
        ),
        preset(
            SourcePresetKind::ColorPipeline,
            "finishing-color",
            color(SourceColorField::BasicExposure),
            "0stops",
        ),
        node(
            SourceNodeRef::preset_audio_processor("main.veac", "voice-chain", "final-limiter"),
            audio(
                SourceAudioProcessorKind::Limiter,
                SourceAudioProcessorField::Ceiling,
            ),
            "-1db",
        ),
        node(
            SourceNodeRef::preset_audio_eq_band(
                "main.veac",
                "voice-chain",
                "tone-shaper",
                "presence",
            ),
            ExpressionSite::PresetAudioEqBandField {
                field: SourceAudioEqBandField::Gain,
            },
            "1db",
        ),
        node(
            SourceNodeRef::preset_delivery_artifact(
                "main.veac",
                "wave-delivery",
                SourceDeliveryArtifactKind::AudioStem,
                "voice-stem",
            ),
            delivery(SourceDeliveryField::Target),
            "\"before.wav\"",
        ),
    ];
    for (target, site, expected) in cases {
        assert_source(&inventory, target, site, expected);
    }
    assert!(inventory
        .nodes
        .windows(2)
        .all(|pair| pair[0].target < pair[1].target));
}
