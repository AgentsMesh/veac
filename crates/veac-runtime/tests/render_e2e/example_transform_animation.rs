use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use super::support::*;
use tempfile::TempDir;
use veac_ir::{Animatable, ProjectEnvelope};
use veac_lang::program::build_path;

fn example_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/transforms-and-animation")
}

fn example_project() -> ProjectEnvelope {
    let built = build_path(&example_root().join("main.veac")).expect("example must execute");
    let envelope = built.envelope();
    let badge = envelope.project.sequences[0].tracks[1]
        .clips
        .iter()
        .find(|clip| {
            clip.authorship
                .as_ref()
                .and_then(|value| value.logical_path.last())
                .map(|value| value.as_str())
                == Some("badge")
        })
        .expect("authored badge");
    assert!(matches!(
        badge.visual.as_ref().unwrap().opacity,
        Animatable::Binding { .. }
    ));
    assert!(envelope
        .temporal
        .provenance
        .iter()
        .any(|value| value.origin.function == "animate"));
    envelope.clone()
}

fn frame_at(path: &Path, seconds: &str) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-ss", seconds, "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
        ])
        .output()
        .expect("ffmpeg must extract the final frame");
    assert!(output.status.success(), "ffmpeg frame extraction failed");
    assert_eq!(output.stdout.len(), 1280 * 720 * 3);
    output.stdout
}

fn coral_count(frame: &[u8]) -> usize {
    frame
        .chunks_exact(3)
        .filter(|value| is_coral([value[0], value[1], value[2]]))
        .count()
}

fn pixel(frame: &[u8], x: usize, y: usize) -> [u8; 3] {
    let offset = (y * 1280 + x) * 3;
    [frame[offset], frame[offset + 1], frame[offset + 2]]
}

fn is_background([r, g, b]: [u8; 3]) -> bool {
    r < 35 && (40..80).contains(&g) && (35..75).contains(&b)
}

fn is_coral([r, g, b]: [u8; 3]) -> bool {
    r > 180 && g < 130 && b < 120
}

#[test]
fn transform_example_renders_crop_flip_and_shear() {
    let mut plan = example_project();
    plan.project.render_configs[0]
        .raster
        .as_mut()
        .unwrap()
        .frame_rate = ratio(10, 1);
    plan.project.sequences[0].settings.frame_rate = ratio(10, 1);
    let temp = TempDir::new().expect("temp dir");
    let output = temp.path().join("transform-example.mkv");
    let font = example_root().join("assets/veac-example-zh.ttf");
    let assets = BTreeMap::from([(plan.project.materials[0].id.to_string(), font)]);
    let rendered = render(plan, &assets, &output);
    let graph = rendered
        .command
        .filter_graph
        .as_deref()
        .expect("video render must have a filter graph");
    assert!(graph.contains("crop="), "graph={graph}");
    assert!(graph.contains("hflip"), "graph={graph}");
    assert!(graph.contains("shear=shx=0.22:shy=-0.08"), "graph={graph}");

    let frame = frame_at(&output, "2.65");
    assert!(is_background(pixel(&frame, 0, 0)));
    assert!(is_background(pixel(&frame, 1270, 710)));

    let coral: Vec<_> = frame
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, value)| is_coral([value[0], value[1], value[2]]))
        .map(|(index, _)| (index % 1280, index / 1280))
        .collect();
    assert!(coral.len() > 1_000, "the transformed badge must be visible");
    let min_x = coral.iter().map(|point| point.0).min().unwrap();
    let min_y = coral.iter().map(|point| point.1).min().unwrap();
    let max_x = coral.iter().map(|point| point.0).max().unwrap();
    let max_y = coral.iter().map(|point| point.1).max().unwrap();
    assert!(
        min_x > 500 && min_y > 250 && max_x < 1240 && max_y < 640,
        "unexpected transformed bounds: {min_x},{min_y}..{max_x},{max_y}"
    );
    assert!(
        frame
            .chunks_exact(3)
            .filter(|value| value[0] > 210 && value[1] > 190 && value[2] > 175)
            .count()
            > 100,
        "the light outline must remain visible"
    );

    let early = coral_count(&frame_at(&output, "0.0"));
    let full = coral_count(&frame_at(&output, "0.3"));
    let late = coral_count(&frame_at(&output, "3.9"));
    assert!(
        full > early.saturating_mul(3) && full > late.saturating_mul(3),
        "authored opacity did not change rendered pixels: {early}/{full}/{late}"
    );
}
