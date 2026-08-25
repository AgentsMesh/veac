use super::support::*;
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const EXECUTABLE: &str = r##"fn rate(divisor: int) -> int { let ignored = 1 / divisor; 10 }
fn main(context: Context) -> Project {
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let scene_item = item(
    identifier("background"), item_enabled(), during(0s, 200ms),
    source_generated(generator_solid(#2b6574ff)), source_timing_native());
  let layer = visual_layer(
    identifier("visual"), 0, placement_free(), state, track_routing_default())
    .with_item(scene_item);
  let timeline = sequence(
    identifier("main"), "CLI frontend",
    sequence_settings(canvas(64px, 36px), frame_rate(rate(0), 1), 48000))
    .with_layer(layer);
  let video_spec = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(23), gop_auto(), b_frames_auto(), video_profile_auto(), video_level_auto());
  let artifact = deliverable_video(
    identifier("main"), delivery_file("frontend.mp4"),
    video_delivery(container_mp4(), video_spec, embedded_audio_none(),
      true, pass_single(), hardware_software()));
  let delivery_spec = delivery(
    identifier("main"), timeline,
    raster_settings(canvas(64px, 36px), frame_rate(10, 1), caption_discard()),
    [artifact]);
  project(identifier("frontend"), project_settings(600))
    .with_sequence(timeline).entry(timeline).with_delivery(delivery_spec)
}
"##;

#[test]
fn source_commands_accept_only_executable_entries() {
    let temp = tempdir().unwrap();
    let executable = source_file(&temp, &EXECUTABLE.replace("rate(0)", "rate(1)"));
    veac()
        .args(["check", executable.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));

    let non_executable = source_file(&temp, "sequence main {}\n");
    veac()
        .args(["check", non_executable.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROGRAM_DECLARATION"));
}

#[test]
fn legacy_frontend_switches_and_compile_command_are_rejected() {
    veac()
        .args(["check", "main.veac", "--frontend", "executable"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument '--frontend'"));
    veac()
        .args(["compile", "main.veac"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unrecognized subcommand 'compile'",
        ));
}

#[test]
fn executable_revision_index_and_format_only_prepare_the_source_graph() {
    let temp = tempdir().unwrap();
    let formatted = veac_lang::program::format_source(EXECUTABLE).unwrap();
    let source = source_file(&temp, &formatted);
    let before = std::fs::read(&source).unwrap();
    for command in ["source-revision", "source-index"] {
        veac()
            .args([command, source.to_str().unwrap()])
            .assert()
            .success();
    }
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
    assert_eq!(std::fs::read(&source).unwrap(), before);
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROGRAM_EXECUTABLE_RUNTIME"));
}

#[test]
fn executable_source_edit_dry_run_repairs_runtime_without_writing() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, EXECUTABLE);
    let before = std::fs::read_to_string(&source).unwrap();
    let prepared = veac_lang::program::prepare_path(&source).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_cli_executable_repair").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "rate"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 24 }".into(),
        },
    });
    let batch_path = temp.path().join("edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();
    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dry_run\": true"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), before);
    assert!(temp.path().join(".veac-source.lock").is_file());
}
