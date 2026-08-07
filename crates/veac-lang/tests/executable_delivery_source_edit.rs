use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, prepare_path, SourceTransactionError};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const SOURCE: &str = r#"
fn movie_target() -> DeliverableTarget { delivery_file("first.mp4") }
fn main(context: Context) -> Project {
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let clip = item(
    identifier("picture"), item_enabled(), during(0s, 2s),
    source_generated(generator_solid(#285577ff)), source_timing_native());
  let layer = visual_layer(
    identifier("visual"), 0, placement_free(), state, track_routing_default())
    .with_item(clip);
  let timeline = sequence(
    identifier("main"), "交付源码编辑",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  let video = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(23), gop_auto(), b_frames_auto(),
    video_profile_auto(), video_level_auto());
  let artifact = deliverable_video(
    identifier("movie"), movie_target(),
    video_delivery(container_mp4(), video, embedded_audio_none(),
      false, pass_single(), hardware_software()));
  let output = delivery(
    identifier("default"), timeline,
    raster_settings(canvas(320px, 180px), frame_rate(30, 1), caption_discard()),
    [artifact]);
  project(identifier("delivery-edit"), project_settings(600))
    .with_sequence(timeline).entry(timeline).with_delivery(output)
}
"#;

#[test]
fn delivery_target_edit_rebuilds_canonical_preview_without_writing_source() {
    let fixture = fixture();
    let before = fs::read(&fixture).unwrap();
    let preview = apply_executable_source_edit_path(
        &fixture,
        &batch(
            &fixture,
            r#"{ delivery_file("second.mp4") }"#,
            "op_delivery_edit",
        ),
    )
    .unwrap();
    let target = &preview.built.envelope().project.render_configs[0].deliverables[0].target;
    assert_eq!(target.file_name(), Some("second.mp4"));
    assert!(preview.source().unwrap().contains("second.mp4"));
    assert_eq!(fs::read(&fixture).unwrap(), before);
}

#[test]
fn incompatible_delivery_edit_is_atomic() {
    let fixture = fixture();
    let before = fs::read(&fixture).unwrap();
    let error = apply_executable_source_edit_path(
        &fixture,
        &batch(
            &fixture,
            r#"{ delivery_file("second.mov") }"#,
            "op_delivery_bad",
        ),
    )
    .unwrap_err();
    assert!(matches!(error, SourceTransactionError::Program(_)));
    assert!(error.to_string().contains("EXECUTABLE_LOWER_IR_VALIDATION"));
    assert_eq!(fs::read(&fixture).unwrap(), before);
}

fn fixture() -> std::path::PathBuf {
    let directory = tempdir().unwrap().keep();
    let entry = directory.join("main.veac");
    fs::write(&entry, SOURCE).unwrap();
    entry
}

fn batch(entry: &std::path::Path, body: &str, operation: &str) -> SourceEditBatch {
    let prepared = prepare_path(entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new(operation).unwrap(),
        prepared.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "movie_target"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: body.to_owned(),
        },
    });
    batch
}
