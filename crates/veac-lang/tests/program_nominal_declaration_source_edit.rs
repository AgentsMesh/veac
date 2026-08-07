use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, prepare_path};
use veac_lang::source_edit::{
    BodySite, BodySource, DeclarationSite, DeclarationSource, SourceEditBatch, SourceEditOperation,
    SourceNodeRef, SourcePrecondition,
};

const SOURCE: &str = r#"struct Timing { duration: time, }
enum Choice { Exact { timing: Timing, }, Fallback, }
fn choose(value: Choice) -> time {
  match value {
    Choice.Exact { timing } => timing.duration,
    Choice.Fallback => 1s,
  }
}
fn selected() -> Choice { Choice.Exact { timing: Timing { duration: 2s, }, } }
fn main(context: Context) -> Project {
  let result = item(identifier("result"), item_enabled(), during(0s, choose(selected())),
    source_generated(generator_transparent()), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(result);
  let timeline = sequence(identifier("main"), "名义声明源码编辑",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("nominal-declaration-edit"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}"#;

#[test]
fn atomic_declaration_replacements_retype_reexecute_and_lower() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let compiled = prepare_path(&entry).unwrap();
    let index = compiled.source_index().unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_nominal_declaration_edit").unwrap(),
        index.revision().clone(),
    );
    add_declaration_edit(
        &mut batch,
        SourceNodeRef::struct_field("main.veac", "Timing", "duration"),
        DeclarationSite::StructFieldDeclaration,
        "duration: time",
        "amount: time",
    );
    add_declaration_edit(
        &mut batch,
        SourceNodeRef::enum_variant("main.veac", "Choice", "Exact"),
        DeclarationSite::EnumVariantDeclaration,
        "Exact { timing: Timing, }",
        "Chosen { timing: Timing, }",
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "choose"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: r#"{
  match value {
    Choice.Chosen { timing } => timing.amount,
    Choice.Fallback => 1s,
  }
}"#
            .into(),
        },
    });
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "selected"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ Choice.Chosen { timing: Timing { amount: 2s, }, } }".into(),
        },
    });

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(duration(preview.built.envelope()).value, 1_200);
    assert!(preview.source().unwrap().contains("amount: time"));
    assert!(preview.source().unwrap().contains("Choice.Chosen"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), SOURCE);
    veac_ir::validate(preview.built.envelope()).unwrap();
}

#[test]
fn rejected_declaration_preview_never_changes_the_source_file() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let compiled = prepare_path(&entry).unwrap();
    let index = compiled.source_index().unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_rejected_nominal_declaration").unwrap(),
        index.revision().clone(),
    );
    add_declaration_edit(
        &mut batch,
        SourceNodeRef::struct_field("main.veac", "Timing", "duration"),
        DeclarationSite::StructFieldDeclaration,
        "duration: time",
        "amount: time",
    );
    assert!(apply_executable_source_edit_path(&entry, &batch).is_err());
    assert_eq!(fs::read_to_string(&entry).unwrap(), SOURCE);
}

#[test]
fn missing_declaration_target_is_rejected_without_writing() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let compiled = prepare_path(&entry).unwrap();
    let index = compiled.source_index().unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_missing_nominal_declaration").unwrap(),
        index.revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetDeclaration {
        target: SourceNodeRef::struct_field("main.veac", "Timing", "missing"),
        site: DeclarationSite::StructFieldDeclaration,
        declaration: DeclarationSource {
            source: "replacement: time".into(),
        },
    });
    let error = apply_executable_source_edit_path(&entry, &batch).unwrap_err();
    assert!(error.to_string().contains("operation 0"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), SOURCE);
}

fn add_declaration_edit(
    batch: &mut SourceEditBatch,
    target: SourceNodeRef,
    site: DeclarationSite,
    before: &str,
    after: &str,
) {
    batch
        .preconditions
        .push(SourcePrecondition::DeclarationEquals {
            target: target.clone(),
            site,
            declaration: DeclarationSource {
                source: before.into(),
            },
        });
    batch.operations.push(SourceEditOperation::SetDeclaration {
        target,
        site,
        declaration: DeclarationSource {
            source: after.into(),
        },
    });
}

fn duration(envelope: &veac_ir::ProjectEnvelope) -> veac_ir::RationalTime {
    envelope.project.sequences[0].tracks[0].clips[0]
        .record_range
        .duration
}
