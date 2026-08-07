use std::path::{Path, PathBuf};
use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::tempdir;

const SOURCE: &str = r#"fn state() -> TrackState {
  track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  )
}

fn flipped() -> VisualStyle {
  visual_style(
    visual_layout(
      placement_anchor(anchor_center(), vector(0.0, 0.0)),
      frame_sized(64px, 36px, fit_fill()),
      transform_2d(
        transform_motion(
          point_constant(point(0px, 0px)),
          vector_constant(vector(1.0, 1.0)), angle_constant(0deg)
        ),
        transform_geometry(
          vector(0.0, 0.0), flip_horizontal(), vector(0.5, 0.5), crop_none()
        )
      )
    ),
    visual_surface(
      percent_constant(100%), compositing(0, blend_normal()), card_none()
    ),
    [], color_pipeline_none()
  )
}

fn main(context: Context) -> Project {
  let image = image_resource(
    identifier("fixture"), resource_file("assets/executable-local-image.png"),
    sha256("e8e9cbd056dda50d6ef53b8b73946e0d238d9dead67be2a2259113bbf9e043e5")
  );
  let scene_item = item(
    identifier("fixture"), item_enabled(), during(0s, 1s),
    source_media(image), source_timing_native()
  ).with_visual(flipped());
  let layer = visual_layer(
    identifier("visual"), 0, placement_free(), state(), track_routing_default()
  ).with_item(scene_item);
  let timeline = sequence(
    identifier("main"), "本地图片水平翻转",
    sequence_settings(canvas(64px, 36px), frame_rate(10, 1), 48000)
  ).with_layer(layer);
  let video_spec = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(18), gop_auto(), b_frames_auto(),
    video_profile_present(profile_h264_high()), video_level_auto()
  );
  let artifact = deliverable_video(
    identifier("preview"), delivery_file("preview.mp4"),
    video_delivery(
      container_mp4(), video_spec, embedded_audio_none(),
      true, pass_single(), hardware_software()
    )
  );
  let delivery_spec = delivery(
    identifier("preview"), timeline,
    raster_settings(canvas(64px, 36px), frame_rate(10, 1), caption_discard()),
    [artifact]
  );
  project(identifier("image-e2e"), project_settings(600))
    .with_resource(image).with_sequence(timeline).entry(timeline).with_delivery(delivery_spec)
}
"#;

#[test]
#[ignore = "isolated FFmpeg guard; run make e2e-executable-local-image"]
fn executable_local_image_builds_plans_and_renders_observable_pixels() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("main.veac");
    let asset_dir = temp.path().join("assets");
    let project = temp.path().join("project.json");
    let rendered = temp.path().join("rendered");
    std::fs::create_dir_all(&asset_dir).unwrap();
    std::fs::create_dir(&rendered).unwrap();
    std::fs::write(&source, SOURCE).unwrap();
    assert_eq!(
        veac_runtime::asset::sha256_identity(&fixture())
            .unwrap()
            .digest,
        "e8e9cbd056dda50d6ef53b8b73946e0d238d9dead67be2a2259113bbf9e043e5"
    );
    std::fs::copy(fixture(), asset_dir.join("executable-local-image.png")).unwrap();

    veac()
        .args(["build", source.to_str().unwrap(), "--emit-ir"])
        .arg(&project)
        .assert()
        .success();
    let plan = veac().args(["plan"]).arg(&project).output().unwrap();
    assert!(
        plan.status.success(),
        "{}",
        String::from_utf8_lossy(&plan.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert_eq!(
        plan["inputs"][0]["canonical_uri"],
        "assets/executable-local-image.png"
    );

    veac()
        .args(["render"])
        .arg(&project)
        .args(["--destination"])
        .arg(&rendered)
        .assert()
        .success();
    let output = rendered.join("preview.mp4");
    assert_probe(&output);
    assert_pixels(&output);
}

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/executable-local-image/assets/executable-local-image.png")
}

fn veac() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("veac"))
}

fn assert_probe(path: &Path) {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=width,height:format=duration",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["streams"][0]["width"], 64);
    assert_eq!(value["streams"][0]["height"], 36);
    let duration = value["format"]["duration"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!((0.9..=1.1).contains(&duration));
}

fn assert_pixels(path: &Path) {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-ss", "0.5", "-i"])
        .arg(path)
        .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout.len(), 64 * 36 * 3);
    let left = region_luma(&output.stdout, 4..28);
    let right = region_luma(&output.stdout, 36..60);
    assert!(left < 35, "flipped left luma was {left}");
    assert!(right > 220, "flipped right luma was {right}");
    assert!(right.saturating_sub(left) > 180);
}

fn region_luma(frame: &[u8], columns: std::ops::Range<usize>) -> u8 {
    let mut total = 0usize;
    let mut samples = 0usize;
    for y in 4..32 {
        for x in columns.clone() {
            let offset = (y * 64 + x) * 3;
            total += frame[offset..offset + 3]
                .iter()
                .map(|value| usize::from(*value))
                .sum::<usize>()
                / 3;
            samples += 1;
        }
    }
    u8::try_from(total / samples).unwrap()
}
