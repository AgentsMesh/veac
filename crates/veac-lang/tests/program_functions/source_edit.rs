use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path, BuiltProgram};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef, SourcePrecondition,
};

use super::support::{item_duration, validate_canonical};

const GRAPH_BODY: &str = r#"
fn timed(key: identifier, duration: time) -> Item {
  item(key, item_enabled(), during(0s, duration),
    source_generated(generator_transparent()), source_timing_native())
}
fn main(context: Context) -> Project {
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("content"), 0, placement_free(), state,
    track_routing_default())$ITEMS;
  let timeline = sequence(identifier("main"), "函数编辑",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  project(identifier("function-edit"), project_settings(1000))
    .with_sequence(timeline).entry(timeline)
}"#;

fn source() -> String {
    let declarations = r#"fn offset(value: time) -> time { value + 1s }"#;
    let items = r#"
    .with_item(timed(identifier("direct"), offset(1s)))
    .with_item(timed(identifier("inline"), offset(4s)))
    .with_item(timed(identifier("default"), offset(offset(2s))))
    .with_item(timed(identifier("bound"), offset(offset(3s))))"#;
    format!("{declarations}\n{}", GRAPH_BODY.replace("$ITEMS", items))
}

const IMPORTED_MODULE: &str = r#"module {
  export fn stretch(value: time) -> time { value * 2.0 }
}"#;

fn imported_entry() -> String {
    let declarations = r#"import "./timing.veac" as timing;"#;
    let items = r#"
    .with_item(timed(identifier("direct"), timing.stretch(1s)))
    .with_item(timed(identifier("inline"), timing.stretch(2s)))"#;
    format!("{declarations}\n{}", GRAPH_BODY.replace("$ITEMS", items))
}

#[test]
fn editing_a_function_body_reexecutes_every_call_site() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let original = source();
    fs::write(&entry, &original).unwrap();
    let built = build_path(&entry).unwrap();
    assert_eq!(durations(&built), ["2s", "5s", "4s", "5s"]);

    let target = SourceNodeRef::function("main.veac", "offset");
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_function_integration").unwrap(),
        built.source_revision().unwrap(),
    );
    batch.preconditions.push(SourcePrecondition::BodyEquals {
        target: target.clone(),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ value + 1s }".into(),
        },
    });
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ value + 2s }".into(),
        },
    });

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(durations(&preview.built), ["3s", "6s", "6s", "7s"]);
    assert!(preview.source().unwrap().contains("value + 2s"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), original);
    assert_ne!(preview.previous_revision, preview.new_revision);
    validate_canonical(&preview.built);
}

#[test]
fn editing_an_imported_function_is_a_revision_bound_preview() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let module = temp.path().join("timing.veac");
    let entry_source = imported_entry();
    fs::write(&entry, &entry_source).unwrap();
    fs::write(&module, IMPORTED_MODULE).unwrap();
    let built = build_path(&entry).unwrap();
    assert_eq!(imported_durations(&built), ["2s", "4s"]);

    let revision = built.source_revision().unwrap();
    let target = SourceNodeRef::function("timing.veac", "stretch");
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_imported_function").unwrap(),
        revision.clone(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target,
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ value * 3.0 }".into(),
        },
    });

    let preview = apply_executable_source_edit_path(&entry, &batch).unwrap();
    assert_eq!(preview.changed_modules(), ["timing.veac"]);
    assert_eq!(preview.previous_revision, revision);
    assert_ne!(preview.new_revision, preview.previous_revision);
    assert_eq!(imported_durations(&preview.built), ["3s", "6s"]);
    assert_eq!(preview.previous_source(), Some(IMPORTED_MODULE));
    assert_eq!(
        preview.source().unwrap(),
        IMPORTED_MODULE.replace("value * 2.0", "value * 3.0")
    );
    assert_eq!(preview.built.sources()["main.veac"], entry_source);
    assert_eq!(fs::read_to_string(&module).unwrap(), IMPORTED_MODULE);
    validate_canonical(&preview.built);
}

fn durations(program: &BuiltProgram) -> [String; 4] {
    std::array::from_fn(|clip| item_duration(program, 0, 0, clip))
}

fn imported_durations(program: &BuiltProgram) -> [String; 2] {
    std::array::from_fn(|clip| item_duration(program, 0, 0, clip))
}
