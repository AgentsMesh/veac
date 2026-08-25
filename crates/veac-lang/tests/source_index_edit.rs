use std::fs;

use tempfile::tempdir;
use veac_ir::{Animatable, EffectParameter};
use veac_lang::program::{apply_executable_source_edit_path, prepare_path, SourceTransactionError};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const SOURCE: &str = r#"
fn amount() -> scalar { 1.0 }

fn card(key: identifier, start: time, color: color) -> Item {
  item(
    key, item_enabled(), during(start, 1s),
    source_generated(generator_solid(color)), source_timing_native()
  ).with_effect(video_sharpen_effect(
    identifier("shared"), effect_enabled(effect_window_full()),
    scalar_constant(amount())
  ))
}

fn main(context: Context) -> Project {
  let first = card(identifier("first"), 0s, #245b78ff);
  let second = card(identifier("second"), 1s, #8d315bff);
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(first).with_item(second);
  let timeline = sequence(identifier("main"), "源码索引",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("source-index-edit"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn function_body_edit_rebuilds_canonical_ir_without_writing_source() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let before = fs::read(&entry).unwrap();
    let prepared = prepare_path(&entry).unwrap();
    let target = SourceNodeRef::function("main.veac", "amount");
    assert!(prepared
        .source_index()
        .unwrap()
        .body(&target, BodySite::FunctionBody)
        .is_some());

    let preview = apply_executable_source_edit_path(&entry, &batch(&entry, "{ 2.0 }")).unwrap();
    let clips = &preview.built.envelope().project.sequences[0].tracks[0].clips;
    assert_eq!(clips.len(), 2);
    assert_ne!(clips[0].effects[0].id, clips[1].effects[0].id);
    for clip in clips {
        assert_eq!(
            clip.effects[0].effect.curve(EffectParameter::Amount),
            Some(&Animatable::constant(2.0))
        );
    }
    assert!(preview
        .source()
        .unwrap()
        .contains("fn amount() -> scalar { 2.0 }"));
    assert_eq!(fs::read(&entry).unwrap(), before);
    veac_ir::validate(preview.built.envelope()).unwrap();
}

#[test]
fn invalid_function_body_edit_is_atomic() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let before = fs::read(&entry).unwrap();
    let error =
        apply_executable_source_edit_path(&entry, &batch(&entry, "{ \"wrong\" }")).unwrap_err();
    assert!(matches!(error, SourceTransactionError::Program(_)));
    assert!(
        error.to_string().contains("EXPRESSION_RETURN_TYPE"),
        "{error}"
    );
    assert_eq!(fs::read(&entry).unwrap(), before);
}

fn batch(entry: &std::path::Path, body: &str) -> SourceEditBatch {
    let prepared = prepare_path(entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_source_index_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "amount"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    batch
}
