use std::fs;

use tempfile::tempdir;
use veac_ir::{Animatable, LengthUnit};
use veac_lang::program::{apply_executable_source_edit_path, prepare_path, SourceTransactionError};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const SOURCE: &str = r#"
fn motion() -> Transform2D {
  transform_2d(
    transform_motion(
      point_constant(point(10px, 0px)), vector_constant(vector(1.0, 1.0)),
      angle_constant(0deg)
    ),
    transform_geometry(vector(0.0, 0.0), flip_none(), vector(0.5, 0.5), crop_none())
  )
}
fn appearance(value: Transform2D) -> VisualStyle {
  visual_style(
    visual_layout(placement_anchor(anchor_center(), vector(0.0, 0.0)),
      frame_none(), value),
    visual_surface(percent_constant(100%), compositing(0, blend_normal()), card_none()),
    [], color_pipeline_none()
  )
}
fn main(context: Context) -> Project {
  let clip = item(identifier("card"), item_enabled(), during(0s, 1s),
    source_generated(generator_transparent()), source_timing_native())
    .with_visual(appearance(motion()));
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(clip);
  let timeline = sequence(identifier("main"), "变换源码编辑",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("edit"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn transform_body_edit_rebuilds_from_source_truth_without_writing() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let before = fs::read_to_string(&entry).unwrap();
    let preview = apply_executable_source_edit_path(
        &entry,
        &batch(
            &entry,
            "{ transform_2d(transform_motion(point_constant(point(50px, -12px)), \
             vector_constant(vector(1.0, 1.0)), angle_constant(0deg)), \
             transform_geometry(vector(0.0, 0.0), flip_horizontal(), \
             vector(0.5, 0.5), crop_none())) }",
        ),
    )
    .unwrap();
    let transform = &preview.built.envelope().project.sequences[0].tracks[0].clips[0]
        .visual
        .as_ref()
        .unwrap()
        .transform;
    let Animatable::Constant { value: position } = &transform.position else {
        panic!("edited transform must stay constant")
    };
    assert_eq!(position.x.value, 50.0);
    assert_eq!(position.x.unit, LengthUnit::Pixels);
    assert_eq!(position.y.value, -12.0);
    assert!(transform.flip_horizontal);
    assert!(preview.source().unwrap().contains("point(50px, -12px)"));
    assert_eq!(fs::read_to_string(&entry).unwrap(), before);
}

#[test]
fn invalid_transform_edit_is_atomic_after_executable_lowering() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    let before = fs::read_to_string(&entry).unwrap();
    let error = apply_executable_source_edit_path(
        &entry,
        &batch(
            &entry,
            "{ transform_2d(transform_motion(\
          point_constant(point(0px, 0px)), vector_constant(vector(0.0, 1.0)), \
          angle_constant(0deg)), transform_geometry(vector(0.0, 0.0), flip_none(), \
          vector(0.5, 0.5), crop_none())) }",
        ),
    )
    .unwrap_err();
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
        veac_ir::OperationId::new("op_transform_source_edit").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "motion"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    batch
}
