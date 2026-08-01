use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use super::support::*;
use tempfile::TempDir;
use veac_ir::ProjectEnvelope;
use veac_lang::{lower_document, parse};

const SOURCE: &str = include_str!("../../../../examples/transforms-and-animation/main.veac");

fn example_project() -> ProjectEnvelope {
    let document = parse(SOURCE).expect("example must parse");
    lower_document(&document).expect("example must lower")
}

fn final_frame(path: &Path) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-ss", "3.0", "-i"])
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
    let mut render_config = project(false).project.render_configs.remove(0);
    render_config.sequence_id = plan.project.sequences[0].id.clone();
    let raster = render_config.raster.as_mut().expect("raster fixture");
    raster.width = 1280;
    raster.height = 720;
    raster.frame_rate = ratio(1, 1);
    plan.project.sequences[0].settings.frame_rate = ratio(1, 1);
    plan.project.render_configs.push(render_config);
    let temp = TempDir::new().expect("temp dir");
    let output = temp.path().join("transform-example.mkv");
    let rendered = render(plan, &BTreeMap::new(), &output);
    let graph = rendered
        .command
        .filter_graph
        .as_deref()
        .expect("video render must have a filter graph");
    assert!(graph.contains("crop="), "graph={graph}");
    assert!(graph.contains("hflip"), "graph={graph}");
    assert!(graph.contains("shear=shx=0.22:shy=-0.08"), "graph={graph}");

    let frame = final_frame(&output);
    assert!(is_background(pixel(&frame, 0, 0)));
    assert!(is_background(pixel(&frame, 1270, 710)));
    assert!(is_coral(pixel(&frame, 880, 500)));
    assert!(is_background(pixel(&frame, 880, 560)));
    assert!(is_coral(pixel(&frame, 1190, 480)));
    assert!(is_background(pixel(&frame, 1190, 550)));
    assert!(is_coral(pixel(&frame, 1170, 570)));

    let coral: Vec<_> = frame
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, value)| is_coral([value[0], value[1], value[2]]))
        .map(|(index, _)| (index % 1280, index / 1280))
        .collect();
    let max_x = coral.iter().map(|point| point.0).max().unwrap();
    let max_y = coral.iter().map(|point| point.1).max().unwrap();
    assert!(
        max_x < 1240 && max_y < 640,
        "unsafe bounds: {max_x}x{max_y}"
    );
    assert!(
        frame
            .chunks_exact(3)
            .filter(|value| value[0] > 210 && value[1] > 190 && value[2] > 175)
            .count()
            > 100,
        "the light outline must remain visible"
    );
}
