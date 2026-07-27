use std::path::Path;
use std::process::Command as ProcessCommand;

use super::support::*;

#[test]
fn public_generated_example_reaches_a_real_observable_delivery() {
    let temp = tempdir().unwrap();
    let source = std::fs::read_to_string(example_path()).unwrap();
    let project = compile_ir(&temp, &source);
    let plan_output = veac()
        .args(["plan", project.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        plan_output.status.success(),
        "{}",
        String::from_utf8_lossy(&plan_output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&plan_output.stdout).unwrap();
    assert_plan_mechanisms(&plan);

    veac()
        .args([
            "render",
            project.to_str().unwrap(),
            "--destination",
            temp.path().to_str().unwrap(),
        ])
        .assert()
        .success();
    let delivery = temp.path().join("executable-mechanisms.mp4");
    assert_delivery_contract(&delivery);
    assert_observable_pixels(&delivery);
    assert!(peak_audio_sample(&delivery) <= 2);
}

fn example_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/executable-mechanisms/main.veac")
}

fn assert_plan_mechanisms(plan: &serde_json::Value) {
    let tracks = &plan["sequences"][0]["tracks"];
    assert_eq!(tracks[0]["transitions"][0]["kind"]["type"], "dissolve");
    let graphic = &tracks[1]["clips"][0];
    assert_eq!(graphic["source"]["generator"]["type"], "shape");
    assert_eq!(graphic["visual"]["compositing"]["blend_mode"], "screen");
    assert_eq!(graphic["visual"]["masks"][0]["shape"]["type"], "circle");
    assert_eq!(
        graphic["visual"]["transform"]["position"]["type"],
        "keyframes"
    );
    assert_eq!(
        tracks[2]["clips"][0]["source"]["generator"]["type"],
        "silence"
    );
}

fn assert_delivery_contract(path: &Path) {
    let output = ProcessCommand::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type,width,height,sample_rate,channels:format=duration",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["streams"][0]["codec_type"], "video");
    assert_eq!(value["streams"][0]["width"], 96);
    assert_eq!(value["streams"][0]["height"], 54);
    assert_eq!(value["streams"][1]["codec_type"], "audio");
    assert_eq!(value["streams"][1]["sample_rate"], "48000");
    assert_eq!(value["streams"][1]["channels"], 1);
    assert_eq!(value["format"]["duration"], "2.000000");
}

fn assert_observable_pixels(path: &Path) {
    let warm_left = rgb_at(path, 0.1, 4, 27);
    let warm_right = rgb_at(path, 0.1, 91, 27);
    assert!(warm_left[0] > 220 && warm_left[1] < 50 && warm_left[2] < 30);
    assert!(warm_right[0] > 220 && warm_right[1] > 200 && warm_right[2] < 30);

    let badge = rgb_at(path, 0.7, 43, 27);
    let before_badge = rgb_at(path, 0.1, 43, 27);
    assert!(badge[1] > before_badge[1].saturating_add(60));

    let transition = rgb_at(path, 1.0, 4, 27);
    assert!(transition[0] > 80 && transition[2] > 80);
    let cool_left = rgb_at(path, 1.9, 4, 27);
    let cool_right = rgb_at(path, 1.9, 91, 27);
    assert!(cool_left[2] > 220 && cool_left[0] < 30);
    assert!(cool_right[1] > 200 && cool_right[2] > 220);
}

fn rgb_at(path: &Path, second: f64, x: u32, y: u32) -> [u8; 3] {
    let output = ProcessCommand::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-ss"])
        .arg(second.to_string())
        .arg("-i")
        .arg(path)
        .args([
            "-vf",
            &format!("crop=2:2:{x}:{y},scale=1:1,format=rgb24"),
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .unwrap();
    assert!(output.status.success() && output.stdout.len() >= 3);
    [output.stdout[0], output.stdout[1], output.stdout[2]]
}

fn peak_audio_sample(path: &Path) -> u16 {
    let output = ProcessCommand::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-map", "0:a:0", "-ac", "1", "-ar", "8000", "-f", "s16le", "-",
        ])
        .output()
        .unwrap();
    assert!(output.status.success() && !output.stdout.is_empty());
    output
        .stdout
        .chunks_exact(2)
        .map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]]).unsigned_abs())
        .max()
        .unwrap()
}
