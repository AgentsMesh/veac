use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;

use super::support::*;

const COMPOSITION_WIDTH: u32 = 1_280;
const COMPOSITION_HEIGHT: u32 = 720;
const DELIVERY_WIDTH: u32 = 240;
const DELIVERY_HEIGHT: u32 = 134;

#[test]
fn composition_canvas_is_preserved_until_square_pixel_delivery_conform() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    let sequence = &mut canonical.project.sequences[0];
    sequence.settings.width = COMPOSITION_WIDTH;
    sequence.settings.height = COMPOSITION_HEIGHT;
    sequence.tracks = canvas_tracks();
    let output = &mut canonical.project.render_configs[0];
    let raster = output.raster.as_mut().expect("raster fixture");
    raster.width = DELIVERY_WIDTH;
    raster.height = DELIVERY_HEIGHT;
    let path = temp.path().join("canvas-delivery.mp4");

    let rendered = render(canonical, &BTreeMap::new(), &path);

    let settings = &rendered.plan.sequences[0].settings;
    assert_eq!((settings.width, settings.height), (1_280, 720));
    let raster = rendered
        .plan
        .output
        .raster
        .as_ref()
        .expect("resolved raster");
    assert_eq!((raster.width, raster.height), (240, 134));
    let stream = video_stream(&path);
    assert_eq!(
        (stream["width"].as_u64(), stream["height"].as_u64()),
        (Some(240), Some(134))
    );
    assert_eq!(stream["sample_aspect_ratio"], "1:1");

    assert_red(rgb_at(&path, 0.5, 120, 67));
    for (x, y) in [(30, 67), (210, 67), (120, 15), (120, 119)] {
        assert_blue(rgb_at(&path, 0.5, x, y));
    }
}

fn canvas_tracks() -> Vec<Track> {
    let background = solid_clip("itm_canvas", color(0, 40, 220), 0, 1_000);
    let mut half_canvas = solid_clip("itm_half_canvas", color(230, 20, 20), 0, 1_000);
    half_canvas.visual = Some(framed_visual(Anchor::Center, Some((640.0, 360.0))));
    vec![
        track("trk_canvas", TrackKind::Video, 0, vec![background]),
        track("trk_half_canvas", TrackKind::Visual, 1, vec![half_canvas]),
    ]
}

fn video_stream(path: &Path) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,sample_aspect_ratio",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .expect("probe conformed delivery");
    assert!(
        output.status.success(),
        "ffprobe failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["streams"][0].clone()
}

fn assert_red(pixel: [u8; 3]) {
    assert!(
        pixel[0] > 170 && pixel[1] < 70 && pixel[2] < 70,
        "pixel={pixel:?}"
    );
}

fn assert_blue(pixel: [u8; 3]) {
    assert!(pixel[2] > 150 && pixel[0] < 70, "pixel={pixel:?}");
}
