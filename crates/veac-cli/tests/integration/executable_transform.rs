use super::support::*;
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const SOURCE: &str = r#"
fn motion() -> Transform2D {
  transform_2d(
    transform_motion(point_constant(point(10px, 0px)),
      vector_constant(vector(1.0, 1.0)), angle_constant(0deg)),
    transform_geometry(vector(0.0, 0.0), flip_none(),
      vector(0.5, 0.5), crop_none()))
}
fn appearance(value: Transform2D) -> VisualStyle {
  visual_style(
    visual_layout(placement_anchor(anchor_center(), vector(0.0, 0.0)),
      frame_none(), value),
    visual_surface(percent_constant(100%), compositing(0, blend_normal()), card_none()),
    [], color_pipeline_none())
}
fn main(context: Context) -> Project {
  let scene_item = item(identifier("card"), item_enabled(), during(0s, 1s),
    source_generated(generator_transparent()), source_timing_native())
    .with_visual(appearance(motion()));
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(scene_item);
  let timeline = sequence(identifier("main"), "CLI transform",
    sequence_settings(canvas(64px, 36px), frame_rate(10, 1), 48000))
    .with_layer(visual);
  project(identifier("cli-transform"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn cli_check_build_and_atomic_source_edit_cover_executable_transform() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, SOURCE);
    let ir = temp.path().join("project.json");
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success();
    veac()
        .args(["build", source.to_str().unwrap(), "--emit-ir"])
        .arg(&ir)
        .assert()
        .success();
    let project: serde_json::Value = serde_json::from_slice(&std::fs::read(ir).unwrap()).unwrap();
    assert_eq!(
        project["project"]["sequences"][0]["tracks"][0]["clips"][0]["visual"]["transform"]
            ["position"]["value"]["x"]["value"],
        10.0
    );

    let before = std::fs::read_to_string(&source).unwrap();
    let edit = edit_file(
        &temp,
        &source,
        "{ transform_2d(transform_motion(point_constant(point(40px, 2px)), \
         vector_constant(vector(1.0, 1.0)), angle_constant(0deg)), \
         transform_geometry(vector(0.0, 0.0), flip_horizontal(), \
         vector(0.5, 0.5), crop_none())) }",
    );
    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            edit.to_str().unwrap(),
        ])
        .arg("--dry-run")
        .assert()
        .success();
    assert_eq!(std::fs::read_to_string(&source).unwrap(), before);
}

fn edit_file(temp: &TempDir, source: &std::path::Path, body: &str) -> std::path::PathBuf {
    let prepared = veac_lang::program::prepare_path(source).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_cli_transform_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "motion"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    let path = temp.path().join("edit.json");
    std::fs::write(
        &path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();
    path
}
