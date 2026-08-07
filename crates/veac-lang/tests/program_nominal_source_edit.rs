use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition,
};

const SOURCE: &str = r#"struct Timing { start: time, duration: time, }
fn finish(value: Timing) -> time { value.start + value.duration }
fn main(context: Context) -> Project {
  let result = item(identifier("result"), item_enabled(),
    during(0s, finish(Timing { duration: 2s, start: 1s, })),
    source_generated(generator_transparent()), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(result);
  let timeline = sequence(identifier("main"), "名义类型源码编辑",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("nominal-source-edit"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}"#;

#[test]
fn editing_a_nominal_function_retypes_and_reexecutes_from_source() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let compiled = build_path(&entry).unwrap();
    assert_eq!(duration(compiled.envelope()).value, 1_800);

    let index = compiled.source_index().unwrap();
    let target = SourceNodeRef::function("main.veac", "finish");
    let original = BodySource {
        source: "{ value.start + value.duration }".into(),
    };
    assert_eq!(
        index.body(&target, BodySite::FunctionBody).unwrap().source,
        original.source
    );
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_nominal_function_body").unwrap(),
        index.revision().clone(),
    );
    batch.preconditions.push(SourcePrecondition::BodyEquals {
        target: target.clone(),
        site: BodySite::FunctionBody,
        body: original,
    });
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ value.duration }".into(),
        },
    });

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(duration(preview.built.envelope()).value, 1_200);
    assert!(preview
        .source()
        .unwrap()
        .contains("fn finish(value: Timing) -> time { value.duration }"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), SOURCE);
    assert_ne!(preview.previous_revision, preview.new_revision);
    veac_ir::validate(preview.built.envelope()).unwrap();
}

fn duration(envelope: &veac_ir::ProjectEnvelope) -> veac_ir::RationalTime {
    envelope.project.sequences[0].tracks[0].clips[0]
        .record_range
        .duration
}
