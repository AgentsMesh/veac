use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const SOURCE: &str = r#"fn offset(value: time) -> time { value + 1s }
fn start() -> time { 0s }
fn main(context: Context) -> Project {
  let sample = item(identifier("sample"), item_enabled(), during(start(), offset(1s)),
    source_generated(generator_transparent()), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("content"), 0, placement_free(), state,
    track_routing_default()).with_item(sample);
  let timeline = sequence(identifier("main"), "原子编辑",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  project(identifier("atomic-body-edit"), project_settings(1000))
    .with_sequence(timeline).entry(timeline)
}"#;

#[test]
fn two_function_body_replacements_share_one_atomic_revision() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let built = build_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_atomic_body_expression").unwrap(),
        built.source_index().unwrap().revision().clone(),
    );
    batch.operations = vec![
        SourceEditOperation::SetBody {
            target: SourceNodeRef::function("main.veac", "offset"),
            site: BodySite::FunctionBody,
            body: BodySource {
                source: "{ value + 2s }".into(),
            },
        },
        SourceEditOperation::SetBody {
            target: SourceNodeRef::function("main.veac", "start"),
            site: BodySite::FunctionBody,
            body: BodySource {
                source: "{ 1s }".into(),
            },
        },
    ];

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert!(preview.source().unwrap().contains("{ value + 2s }"));
    assert!(preview
        .source()
        .unwrap()
        .contains("fn start() -> time { 1s }"));
    let range = preview.built.envelope().project.sequences[0].tracks[0].clips[0].record_range;
    assert_eq!((range.start.value, range.duration.value), (1000, 3000));
    assert_eq!(fs::read_to_string(entry).unwrap(), SOURCE);
}
