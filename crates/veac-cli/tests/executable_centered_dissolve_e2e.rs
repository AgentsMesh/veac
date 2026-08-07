use std::path::Path;
use std::process::Command;

use assert_cmd::prelude::*;
use tempfile::tempdir;

const SOURCE: &str = r#"fn state() -> TrackState {
  track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  )
}

fn solid(key: identifier, fill: color, start: time, duration: time) -> Item {
  item(
    key, item_enabled(), during(start, duration),
    source_generated(generator_solid(fill)), source_timing_native()
  )
}

fn main(context: Context) -> Project {
  let first = solid(identifier("first"), #ef4444ff, 0s, 2s);
  let second = solid(identifier("second"), #2563ebff, 1600ms, 2400ms);
  let cut = relation_transition(
    identifier("cut"), first, second, transition_dissolve(400ms)
  );
  let scenes = visual_layer(
    identifier("scenes"), 0, placement_free(), state(), track_routing_default()
  ).with_item(first).with_item(second);
  let timeline = sequence(
    identifier("main"), "居中叠化",
    sequence_settings(canvas(64px, 36px), frame_rate(10, 1), 48000)
  ).with_layer(scenes).with_relation(cut);
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
  project(identifier("dissolve-e2e"), project_settings(600))
    .with_sequence(timeline).entry(timeline).with_delivery(delivery_spec)
}
"#;

#[test]
#[ignore = "isolated FFmpeg guard; run make e2e-executable-centered-dissolve"]
fn executable_relation_builds_and_renders_monotonic_true_overlap() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("main.veac");
    let project = temp.path().join("project.json");
    let rendered = temp.path().join("rendered");
    std::fs::write(&source, SOURCE).unwrap();
    std::fs::create_dir(&rendered).unwrap();

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
    assert_plan(&serde_json::from_slice(&plan.stdout).unwrap());
    veac()
        .args(["render"])
        .arg(&project)
        .args(["--destination"])
        .arg(&rendered)
        .assert()
        .success();

    let output = rendered.join("preview.mp4");
    let red = pixel(&output, 1.5);
    let overlap = [1.62, 1.7, 1.8, 1.9, 1.98].map(|at| pixel(&output, at));
    let blue = pixel(&output, 2.1);
    assert!(red[0] > 200 && red[2] < 70, "red={red:?}");
    assert!(blue[2] > 180 && blue[0] < 70, "blue={blue:?}");
    assert!(
        overlap[2][0] > 70 && overlap[2][2] > 70,
        "middle={:?}",
        overlap[2]
    );
    for pair in overlap.windows(2) {
        assert!(pair[1][0] <= pair[0][0] + 12, "red channel: {overlap:?}");
        assert!(pair[1][2] + 12 >= pair[0][2], "blue channel: {overlap:?}");
    }
}

fn assert_plan(plan: &serde_json::Value) {
    let transition = plan["sequences"][0]["tracks"][0]["transitions"][0]
        .as_object()
        .unwrap();
    assert_eq!(milliseconds(&transition["record_window"]["start"]), 1_600);
    assert_eq!(milliseconds(&transition["record_window"]["duration"]), 400);
    assert_eq!(milliseconds(&transition["outgoing_range"]["duration"]), 400);
    assert_eq!(milliseconds(&transition["incoming_range"]["duration"]), 400);
}

fn milliseconds(time: &serde_json::Value) -> i64 {
    time["value"].as_i64().unwrap() * 1_000 / time["timescale"].as_i64().unwrap()
}

fn pixel(path: &Path, second: f64) -> [u8; 3] {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-ss",
            &second.to_string(),
            "-vf",
            "crop=2:2:32:18,scale=1:1,format=rgb24",
        ])
        .args(["-frames:v", "1", "-f", "rawvideo", "-"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout[..3].try_into().unwrap()
}

fn veac() -> Command {
    Command::new(assert_cmd::cargo::cargo_bin!("veac"))
}
