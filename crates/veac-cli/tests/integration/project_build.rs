use super::support::*;

const PROJECT: &str = include_str!("../../../veac-project/tests/fixtures/authored_minimal.veac");
pub(super) const TARGET: &str = r#"input parameter profile: text;
input parameter locale: text;
input parameter theme: text;

fn main(context: Context) -> Project {
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let scene_item = item(
    identifier("background"), item_enabled(), during(0s, 200ms),
    source_generated(generator_solid(#112233ff)), source_timing_native());
  let layer = visual_layer(
    identifier("visual"), 0, placement_free(), state, track_routing_default())
    .with_item(scene_item);
  let timeline = sequence(
    identifier("main"), "Project Build E2E",
    sequence_settings(canvas(32px, 24px), frame_rate(10, 1), 48000))
    .with_layer(layer);
  let video_spec = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(23), gop_auto(), b_frames_auto(), video_profile_auto(), video_level_auto());
  let artifact = deliverable_video(
    identifier("video"), delivery_file("render.mp4"),
    video_delivery(container_mp4(), video_spec, embedded_audio_none(),
      true, pass_single(), hardware_software()));
  let delivery_spec = delivery(
    identifier("main"), timeline,
    raster_settings(canvas(32px, 24px), frame_rate(10, 1), caption_discard()),
    [artifact]);
  project(identifier("project-build-e2e"), project_settings(1000))
    .with_sequence(timeline).entry(timeline).with_delivery(delivery_spec)
}
"#;

#[test]
fn project_build_executes_delivers_and_reuses_the_artifact_graph() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let receipt = temp.path().join("build/receipt.json");
    let sources = temp.path().join("sources");
    std::fs::create_dir(&sources).unwrap();
    std::fs::write(&entry, authored_project()).unwrap();
    std::fs::write(sources.join("main.veac"), TARGET).unwrap();
    std::fs::create_dir(temp.path().join("materials")).unwrap();

    let first = invoke(&entry, &receipt);
    let first_receipt_bytes = std::fs::read(&receipt).unwrap();
    assert!(
        first.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&first.stderr),
        String::from_utf8_lossy(&first_receipt_bytes)
    );
    let first_receipt: serde_json::Value = serde_json::from_slice(&first_receipt_bytes).unwrap();
    assert_eq!(first_receipt["outcome"], "succeeded");
    assert_eq!(first_receipt["nodes"][0]["status"], "executed");
    assert_eq!(first_receipt["deliveries"][0]["status"], "published");
    let output = temp.path().join("dist/render/preview/en-us/dark.mp4");
    assert!(output.is_file());

    let second = invoke(&entry, &receipt);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_receipt: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&receipt).unwrap()).unwrap();
    assert_eq!(second_receipt["nodes"][0]["status"], "cache_hit");
    assert_eq!(
        first_receipt["graph_digest"],
        second_receipt["graph_digest"]
    );
}

#[test]
fn project_build_binds_and_verifies_typed_project_materials() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("project.veac");
    let receipt = temp.path().join("build/receipt.json");
    let sources = temp.path().join("sources");
    std::fs::create_dir(&sources).unwrap();
    std::fs::write(&entry, project_with_picture()).unwrap();
    std::fs::write(sources.join("main.veac"), target_with_picture()).unwrap();
    let materials = temp.path().join("materials");
    std::fs::create_dir(&materials).unwrap();
    std::fs::copy(
        project_root().join("examples/video-effects/assets/effects-plate.png"),
        materials.join("plate.png"),
    )
    .unwrap();

    let output = invoke(&entry, &receipt);
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&std::fs::read(&receipt).unwrap())
    );
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&receipt).unwrap()).unwrap();
    assert_eq!(value["outcome"], "succeeded");
    assert!(temp
        .path()
        .join("dist/render/preview/en-us/dark.mp4")
        .is_file());
}

fn invoke(entry: &std::path::Path, receipt: &std::path::Path) -> std::process::Output {
    veac()
        .args(["project", "build"])
        .arg(entry)
        .arg("--receipt")
        .arg(receipt)
        .output()
        .unwrap()
}

pub(super) fn authored_project() -> String {
    PROJECT
        .replace("},\n    ],", "}\n    ],")
        .replace("},\n        ],", "}\n        ],")
}

fn project_with_picture() -> String {
    authored_project().replace(
        "        inputs: [\n",
        "        inputs: [\n          ProjectInput { id: identifier(\"picture\"), source: ProjectInputSource.ProjectMaterial { path: \"plate.png\", }, },\n",
    )
}

pub(super) fn target_with_picture() -> String {
    TARGET
        .replacen(
            "input parameter profile: text;",
            "struct MaterialBinding { kind: text, path: text, sha256: text, authority: text, artifact_key: text, video_stream: int, audio_stream: int, }\ninput material picture: MaterialBinding;\ninput parameter profile: text;",
            1,
        )
        .replacen(
            "  let state =",
            "  let picture_resource = image_resource(identifier(\"picture\"), resource_file(picture.path), sha256(picture.sha256));\n  let state =",
            1,
        )
        .replacen(
            "source_generated(generator_solid(#112233ff))",
            "source_media(picture_resource)",
            1,
        )
        .replacen(
            "    .with_sequence(timeline)",
            "    .with_resource(picture_resource).with_sequence(timeline)",
            1,
        )
}

fn project_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
