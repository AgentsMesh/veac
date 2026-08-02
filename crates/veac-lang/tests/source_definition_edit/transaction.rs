use tempfile::tempdir;
use veac_lang::program::{apply_source_edit_path, compile_path, SourceIndexInventory};
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceAudioEqBandField, SourceAudioProcessorField,
    SourceAudioProcessorKind, SourceColorField, SourceDeliveryArtifactKind, SourceDeliveryField,
    SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition, SourcePresetKind,
    SourceTextLayoutField, SourceTextStyleField,
};

use super::fixture::{expression, SOURCE};

#[test]
fn inventory_targets_edit_definitions_and_recompile_all_use_sites() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("main.veac");
    std::fs::write(&path, SOURCE).unwrap();
    let index = compile_path(&path).unwrap().source_index().unwrap();
    let inventory = index.inventory();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_definition_fields").unwrap(),
        inventory.revision.clone(),
    );
    for (target, site, replacement) in edits() {
        add_edit(&mut batch, &inventory, target, site, replacement);
    }

    let preview = apply_source_edit_path(&path, &batch).unwrap();
    let next = preview.compiled.source_index().unwrap().inventory();
    assert_ne!(preview.previous_revision, preview.new_revision);
    for (target, site, expected) in edits() {
        assert_eq!(expression(&next, &target, &site).source, expected);
    }
    assert!(preview.source().contains("content \"after\""));
    assert!(preview.source().contains("target file \"after.wav\""));
    for expanded in [
        "size 26px",
        "box-width 360px",
        "exposure 1stops",
        "ceiling -2db",
        "gain 3db",
        "target file \"after.wav\"",
    ] {
        assert!(preview.compiled.expanded_source().contains(expanded));
    }
}

fn add_edit(
    batch: &mut SourceEditBatch,
    inventory: &SourceIndexInventory,
    target: SourceNodeRef,
    site: ExpressionSite,
    replacement: &str,
) {
    let current = expression(inventory, &target, &site).source.clone();
    batch
        .preconditions
        .push(SourcePrecondition::ExpressionEquals {
            target: target.clone(),
            site: site.clone(),
            expression: ExpressionSource { source: current },
        });
    batch.operations.push(SourceEditOperation::SetExpression {
        target,
        site,
        expression: ExpressionSource {
            source: replacement.into(),
        },
    });
}

fn edits() -> Vec<(SourceNodeRef, ExpressionSite, &'static str)> {
    vec![
        edit(component_item(), ExpressionSite::ItemRecordDuration, "2s"),
        edit(component_item(), ExpressionSite::TextContent, "\"after\""),
        edit(component_modifier(), parameter("amount"), "1.5"),
        edit(
            SourceNodeRef::component_local_instance("main.veac", "card", "child"),
            ExpressionSite::ComponentLocalInstanceArgument {
                parameter: "duration".into(),
            },
            "750ms",
        ),
        edit(component_stage(), parameter("amount"), "1.2"),
        edit(
            SourceNodeRef::preset("main.veac", SourcePresetKind::TextStyle, "shared"),
            ExpressionSite::PresetTextStyleField {
                field: SourceTextStyleField::Size,
            },
            "26px",
        ),
        edit(
            SourceNodeRef::preset("main.veac", SourcePresetKind::TextLayout, "shared"),
            ExpressionSite::PresetTextLayoutField {
                field: SourceTextLayoutField::BoxWidth,
            },
            "360px",
        ),
        edit(
            SourceNodeRef::preset_modifier("main.veac", "shared-effects", "stacked-sharpen"),
            parameter("amount"),
            "1.5",
        ),
        edit(
            SourceNodeRef::preset_stage("main.veac", "shared-effects", "pipeline-blur"),
            parameter("radius"),
            "3",
        ),
        edit(
            SourceNodeRef::preset(
                "main.veac",
                SourcePresetKind::ColorPipeline,
                "finishing-color",
            ),
            ExpressionSite::PresetColorField {
                field: SourceColorField::BasicExposure,
            },
            "1stops",
        ),
        edit(
            SourceNodeRef::preset_audio_processor("main.veac", "voice-chain", "final-limiter"),
            ExpressionSite::PresetAudioProcessorField {
                processor_kind: SourceAudioProcessorKind::Limiter,
                field: SourceAudioProcessorField::Ceiling,
            },
            "-2db",
        ),
        edit(
            SourceNodeRef::preset_audio_eq_band(
                "main.veac",
                "voice-chain",
                "tone-shaper",
                "presence",
            ),
            ExpressionSite::PresetAudioEqBandField {
                field: SourceAudioEqBandField::Gain,
            },
            "3db",
        ),
        edit(
            SourceNodeRef::preset_delivery_artifact(
                "main.veac",
                "wave-delivery",
                SourceDeliveryArtifactKind::AudioStem,
                "voice-stem",
            ),
            ExpressionSite::PresetDeliveryField {
                field: SourceDeliveryField::Target,
            },
            "\"after.wav\"",
        ),
    ]
}

fn component_item() -> SourceNodeRef {
    SourceNodeRef::component_item("main.veac", "card", "content", "title")
}
fn component_modifier() -> SourceNodeRef {
    SourceNodeRef::component_modifier("main.veac", "card", "content", "title", "sharpen")
}
fn component_stage() -> SourceNodeRef {
    SourceNodeRef::component_stage("main.veac", "card", "finish", "polish")
}
fn parameter(name: &str) -> ExpressionSite {
    ExpressionSite::ModifierParameter {
        parameter: name.into(),
    }
}
fn edit(
    target: SourceNodeRef,
    site: ExpressionSite,
    value: &'static str,
) -> (SourceNodeRef, ExpressionSite, &'static str) {
    (target, site, value)
}
