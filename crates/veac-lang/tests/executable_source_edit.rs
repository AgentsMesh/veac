use std::fs;

use tempfile::tempdir;
use veac_ir::MaterialSource;
use veac_lang::program::{
    apply_executable_source_edit_path, build_path, prepare_path, SourceTransactionError,
};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const ENTRY: &str = r#"import "./timing.veac" as timing;
fn main(context: Context) -> Project {
  let timeline = sequence(identifier("main"), "运行时修复",
    sequence_settings(canvas(320px, 180px), frame_rate(timing.rate(0), 1), 48000));
  project(identifier("edit"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

const RESOURCE_ENTRY: &str = r#"import "./images.veac" as images;
fn main(context: Context) -> Project {
  let image = images.hero();
  let clip = item(identifier("hero"), item_enabled(), during(0s, 1s),
    source_media(image), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(clip);
  let timeline = sequence(identifier("main"), "资源编辑",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("edit"), project_settings(600))
    .with_resource(image).with_sequence(timeline).entry(timeline)
}
"#;

fn batch(entry: &std::path::Path, body: &str) -> SourceEditBatch {
    let prepared = prepare_path(entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_executable_source_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("timing.veac", "rate"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    batch
}

#[test]
fn executable_edit_can_repair_a_runtime_failure_from_source_truth() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let module = temp.path().join("timing.veac");
    fs::write(&entry, ENTRY).unwrap();
    fs::write(
        &module,
        "module { export fn rate(divisor: int) -> int { let ignored = 1 / divisor; 30 } }",
    )
    .unwrap();
    prepare_path(&entry).unwrap();
    assert!(build_path(&entry).is_err());

    let preview = apply_executable_source_edit_path(&entry, &batch(&entry, "{ 30 }")).unwrap();
    assert_eq!(preview.changed_modules(), ["timing.veac"]);
    assert_ne!(preview.previous_revision, preview.new_revision);
    assert_eq!(preview.built.envelope().project.revision, 0);
    assert!(preview
        .source()
        .unwrap()
        .contains("fn rate(divisor: int) -> int { 30 }"));
    assert!(fs::read_to_string(&module).unwrap().contains("1 / divisor"));
}

#[test]
fn failed_executable_overlay_does_not_publish_a_partial_preview() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let module = temp.path().join("timing.veac");
    fs::write(&entry, ENTRY).unwrap();
    fs::write(
        &module,
        "module { export fn rate(divisor: int) -> int { 30 } }",
    )
    .unwrap();
    let before_entry = fs::read_to_string(&entry).unwrap();
    let before_module = fs::read_to_string(&module).unwrap();
    let error = apply_executable_source_edit_path(
        &entry,
        &batch(&entry, "{ let ignored = 1 / divisor; 30 }"),
    )
    .unwrap_err();
    assert!(matches!(error, SourceTransactionError::Program(_)));
    assert_eq!(fs::read_to_string(entry).unwrap(), before_entry);
    assert_eq!(fs::read_to_string(module).unwrap(), before_module);
}

#[test]
fn resource_path_edit_rebuilds_material_from_source_truth() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let module = temp.path().join("images.veac");
    fs::write(&entry, RESOURCE_ENTRY).unwrap();
    fs::write(&module, resource_module("assets/old.png")).unwrap();

    let preview = apply_executable_source_edit_path(
        &entry,
        &resource_batch(
            &entry,
            r#"{ image_resource(identifier("hero"), resource_file("assets/new.png"),
                sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")) }"#,
        ),
    )
    .unwrap();
    assert_eq!(
        preview.built.envelope().project.materials[0].source,
        MaterialSource::File {
            uri: "assets/new.png".to_owned()
        }
    );
    assert!(preview.source().unwrap().contains("assets/new.png"));
    assert!(fs::read_to_string(module)
        .unwrap()
        .contains("assets/old.png"));
}

#[test]
fn invalid_resource_path_edit_is_atomic() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let module = temp.path().join("images.veac");
    fs::write(&entry, RESOURCE_ENTRY).unwrap();
    fs::write(&module, resource_module("assets/old.png")).unwrap();
    let before_entry = fs::read_to_string(&entry).unwrap();
    let before_module = fs::read_to_string(&module).unwrap();

    let error = apply_executable_source_edit_path(
        &entry,
        &resource_batch(
            &entry,
            r#"{ image_resource(identifier("hero"), resource_file("/escape.png"),
                sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")) }"#,
        ),
    )
    .unwrap_err();
    assert!(matches!(error, SourceTransactionError::Program(_)));
    assert_eq!(fs::read_to_string(entry).unwrap(), before_entry);
    assert_eq!(fs::read_to_string(module).unwrap(), before_module);
}

fn resource_batch(entry: &std::path::Path, body: &str) -> SourceEditBatch {
    let prepared = prepare_path(entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_resource_path_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("images.veac", "hero"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    batch
}

fn resource_module(path: &str) -> String {
    format!(
        "module {{ export fn hero() -> Resource {{ \
         image_resource(identifier(\"hero\"), resource_file(\"{path}\"), \
         sha256(\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\")) }} }}"
    )
}
