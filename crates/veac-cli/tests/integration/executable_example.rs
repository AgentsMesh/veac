use std::path::Path;
use std::process::Command as ProcessCommand;

use super::support::*;

#[test]
fn public_generated_example_reaches_a_real_observable_delivery() {
    let temp = tempdir().unwrap();
    let entry = copy_example(&temp);
    let project = temp.path().join("project.json");
    veac()
        .arg("build")
        .arg(entry)
        .args(["--emit-ir"])
        .arg(&project)
        .assert()
        .success();
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
    let delivery = temp.path().join("preview.mp4");
    assert_delivery_contract(&delivery);
    assert_observable_pixels(&delivery);
}

fn example_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/executable-mechanisms/main.veac")
}

fn copy_example(temp: &TempDir) -> std::path::PathBuf {
    let source_root = example_path().parent().unwrap().to_path_buf();
    for name in ["main.veac", "showcase.veac", "annotations.veac"] {
        std::fs::copy(source_root.join(name), temp.path().join(name)).unwrap();
    }
    let assets = temp.path().join("assets");
    std::fs::create_dir(&assets).unwrap();
    std::fs::copy(
        source_root.join("assets/veac-example-zh.ttf"),
        assets.join("veac-example-zh.ttf"),
    )
    .unwrap();
    temp.path().join("main.veac")
}

fn assert_plan_mechanisms(plan: &serde_json::Value) {
    let tracks = plan["sequences"][0]["tracks"].as_array().unwrap();
    let transition = tracks
        .iter()
        .flat_map(|track| track["transitions"].as_array().unwrap())
        .next()
        .unwrap();
    assert_eq!(transition["kind"]["type"], "dissolve");
    let clips = tracks
        .iter()
        .flat_map(|track| track["clips"].as_array().unwrap());
    let graphic = clips
        .clone()
        .find(|clip| clip["source"]["generator"]["type"] == "shape")
        .unwrap();
    assert_eq!(graphic["source"]["generator"]["type"], "shape");
    assert_eq!(graphic["visual"]["compositing"]["blend_mode"], "screen");
    assert_eq!(graphic["visual"]["masks"][0]["shape"]["type"], "circle");
    assert_eq!(
        graphic["visual"]["transform"]["position"]["type"],
        "keyframes"
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
    assert_eq!(value["streams"][0]["width"], 640);
    assert_eq!(value["streams"][0]["height"], 360);
    assert_eq!(value["streams"].as_array().unwrap().len(), 1);
    assert_eq!(value["format"]["duration"], "2.000000");
}

fn assert_observable_pixels(path: &Path) {
    let warm_left = rgb_at(path, 0.1, 27, 180);
    let warm_right = rgb_at(path, 0.1, 607, 180);
    assert!(
        warm_left[0] > 220 && warm_left[1] < 100 && warm_left[2] < 100,
        "unexpected warm-left pixel: {warm_left:?}"
    );
    assert!(
        warm_right[0] > 220 && warm_right[1] > 175 && warm_right[2] < 60,
        "unexpected warm-right pixel: {warm_right:?}"
    );

    let badge = rgb_at(path, 0.7, 67, 133);
    let before_badge = rgb_at(path, 0.1, 67, 133);
    assert!(
        badge[1] > before_badge[1].saturating_add(60),
        "badge {badge:?} did not separate from background {before_badge:?}"
    );

    let transition = rgb_at(path, 0.8, 27, 180);
    assert!(
        transition[0] > 80 && transition[2] > 80,
        "unexpected transition pixel: {transition:?}"
    );
    let cool_left = rgb_at(path, 1.9, 27, 180);
    let cool_right = rgb_at(path, 1.9, 607, 180);
    assert!(
        cool_left[2] > 220 && cool_left[0] < 50,
        "unexpected cool-left pixel: {cool_left:?}"
    );
    assert!(
        cool_right[1] > 200 && cool_right[2] > 220,
        "unexpected cool-right pixel: {cool_right:?}"
    );
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
