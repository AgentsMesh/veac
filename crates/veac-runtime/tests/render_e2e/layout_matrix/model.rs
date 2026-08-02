use std::{collections::BTreeMap, path::PathBuf};

use super::super::support::*;

pub(super) fn layout_clip(
    id: &str,
    start_ms: i64,
    duration_ms: i64,
    placement: Placement,
    fit: FitMode,
) -> Clip {
    let mut clip = solid_clip(id, color(255, 0, 0), start_ms, duration_ms);
    let mut visual = full_visual();
    visual.frame = Some(Frame {
        width: pixels(16.0),
        height: pixels(12.0),
        fit,
    });
    visual.placement = placement;
    clip.visual = Some(visual);
    clip
}

pub(super) fn media_layout_clip(id: &str, start_ms: i64, fit: FitMode) -> Clip {
    let mut clip = media_clip(id, "med_layout", start_ms, 600);
    let mut visual = full_visual();
    visual.frame = Some(Frame {
        width: pixels(32.0),
        height: pixels(32.0),
        fit,
    });
    visual.placement = Placement::Anchor {
        anchor: Anchor::Center,
        inset: Vec2 { x: 0.0, y: 0.0 },
    };
    clip.visual = Some(visual);
    clip
}

pub(super) fn render_layout(project: &ProjectEnvelope) -> (tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("layout.mp4");
    render(project.clone(), &BTreeMap::new(), &output);
    (temp, output)
}

pub(super) fn assert_red(pixel: [u8; 3]) {
    assert!(
        pixel[0] > 150 && pixel[1] < 90 && pixel[2] < 90,
        "pixel={pixel:?}"
    );
}

pub(super) fn assert_black(pixel: [u8; 3]) {
    assert!(pixel.iter().all(|value| *value < 35), "pixel={pixel:?}");
}

pub(super) fn assert_colored(pixel: [u8; 3]) {
    assert!(pixel.iter().any(|value| *value > 45), "pixel={pixel:?}");
}

pub(super) fn length(value: f64, unit: LengthUnit) -> Length {
    Length { value, unit }
}

pub(super) fn pixel_at(frame: &[u8], width: u32, x: u32, y: u32) -> [u8; 3] {
    let offset = ((y * width + x) * 3) as usize;
    [frame[offset], frame[offset + 1], frame[offset + 2]]
}
