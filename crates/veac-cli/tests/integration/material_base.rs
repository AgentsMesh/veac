use super::support::*;

const EXECUTABLE_ENTRY: &str = r#"import "./asset.veac" as asset;
fn main(context: Context) -> Project {
  let poster = asset.poster();
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let scene_item = item(identifier("poster"), item_enabled(), during(0s, 1s),
    source_media(poster), source_timing_native());
  let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(scene_item);
  let timeline = sequence(identifier("main"), "Local material",
    sequence_settings(canvas(64px, 36px), frame_rate(24, 1), 48000)).with_layer(layer);
  let video_spec = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(23), gop_auto(), b_frames_auto(), video_profile_auto(), video_level_auto());
  let artifact = deliverable_video(identifier("main"), delivery_file("local.mp4"),
    video_delivery(container_mp4(), video_spec, embedded_audio_none(), true,
      pass_single(), hardware_software()));
  let delivery_spec = delivery(identifier("main"), timeline,
    raster_settings(canvas(64px, 36px), frame_rate(24, 1), caption_discard()), [artifact]);
  project(identifier("local"), project_settings(600)).with_resource(poster)
    .with_sequence(timeline).entry(timeline).with_delivery(delivery_spec)
}
"#;

const EXECUTABLE_MODULE: &str = r#"module {
  export fn poster() -> Resource {
    image_resource(identifier("poster"), resource_file("assets/poster.bin"),
      sha256("0000000000000000000000000000000000000000000000000000000000000000"))
  }
}
"#;

#[test]
fn generated_only_executable_projects_allow_cross_root_ir() {
    let temp = tempdir().unwrap();
    let other = directory(temp.path(), "other");
    let entry = source_file(&temp, EXECUTABLE_SOURCE);
    assert!(emit(&entry, &other.join("executable.json"))
        .status
        .success());
}

#[test]
fn rejected_output_preserves_target_source_module_and_material_bytes() {
    let temp = tempdir().unwrap();
    let entry = source_file(&temp, EXECUTABLE_ENTRY);
    let module = temp.path().join("asset.veac");
    let assets = directory(temp.path(), "assets");
    let material = assets.join("poster.bin");
    std::fs::write(&module, EXECUTABLE_MODULE).unwrap();
    std::fs::write(&material, b"material").unwrap();
    let other = directory(temp.path(), "other");
    let target = other.join("project.json");
    std::fs::write(&target, b"existing").unwrap();
    let before = [&entry, &module, &material, &target].map(|path| std::fs::read(path).unwrap());

    let output = emit(&entry, &target);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("OUTPUT_MATERIAL_BASE_MISMATCH"));
    let after = [&entry, &module, &material, &target].map(|path| std::fs::read(path).unwrap());
    assert_eq!(after, before);
}

#[cfg(unix)]
#[test]
fn source_root_symlink_alias_preserves_the_material_base() {
    let temp = tempdir().unwrap();
    let entry = source_file(&temp, EXECUTABLE_ENTRY);
    std::fs::write(temp.path().join("asset.veac"), EXECUTABLE_MODULE).unwrap();
    let assets = directory(temp.path(), "assets");
    std::fs::write(assets.join("poster.bin"), b"material").unwrap();
    let alias = temp.path().join("alias");
    std::os::unix::fs::symlink(temp.path(), &alias).unwrap();
    assert!(emit(&entry, &alias.join("project.json")).status.success());
    assert!(temp.path().join("project.json").is_file());
}

fn emit(source: &std::path::Path, target: &std::path::Path) -> std::process::Output {
    veac()
        .arg("build")
        .arg(source)
        .args(["--emit-ir"])
        .arg(target)
        .output()
        .unwrap()
}

fn directory(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    let path = root.join(name);
    std::fs::create_dir(&path).unwrap();
    path
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
