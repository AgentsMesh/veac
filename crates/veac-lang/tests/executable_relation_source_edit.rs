use std::fs;

use tempfile::tempdir;
use veac_ir::RelationKind;
use veac_lang::program::{apply_executable_source_edit_path, prepare_path, SourceTransactionError};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const SOURCE: &str = r#"
fn overlap() -> time { 500ms }
fn main(context: Context) -> Project {
  let outgoing = item(identifier("out"), item_enabled(), during(0s, 1s + overlap()),
    source_generated(generator_transparent()), source_timing_native());
  let incoming = item(identifier("in"), item_enabled(), during(1s, 1s),
    source_generated(generator_transparent()), source_timing_native());
  let cross = relation_transition(
    identifier("cross"), outgoing, incoming, transition_dissolve(overlap()));
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(outgoing).with_item(incoming);
  let timeline = sequence(identifier("main"), "关系源码编辑",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visual).with_relation(cross);
  project(identifier("edit"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn dissolve_duration_edit_rebuilds_relation_from_source_truth() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let before = fs::read_to_string(&entry).unwrap();
    let preview = apply_executable_source_edit_path(&entry, &batch(&entry, "{ 250ms }")).unwrap();
    let RelationKind::Transition { transition, .. } =
        &preview.built.envelope().project.relations[0].kind
    else {
        panic!("expected transition relation")
    };
    assert_eq!(transition.duration.value, 150);
    assert_eq!(transition.duration.timescale, 600);
    assert!(preview
        .source()
        .unwrap()
        .contains("fn overlap() -> time { 250ms }"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), before);
}

#[test]
fn invalid_dissolve_duration_edit_is_atomic_and_does_not_write() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let before = fs::read_to_string(&entry).unwrap();
    let error = apply_executable_source_edit_path(&entry, &batch(&entry, "{ 0ms }")).unwrap_err();
    assert!(matches!(error, SourceTransactionError::Program(_)));
    assert!(
        error.to_string().contains("EXECUTABLE_LOWER_IR_VALIDATION"),
        "{error}"
    );
    assert_eq!(fs::read_to_string(&entry).unwrap(), before);
}

fn batch(entry: &std::path::Path, body: &str) -> SourceEditBatch {
    let prepared = prepare_path(entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_relation_source_edit").unwrap(),
        prepared.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "overlap"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    batch
}
