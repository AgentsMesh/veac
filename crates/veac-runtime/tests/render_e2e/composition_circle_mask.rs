use tempfile::tempdir;

use super::support::*;

fn masked_clip(id: &str, shape: MaskShape, start_ms: i64) -> Clip {
    let mut clip = solid_clip(id, color(255, 0, 0), start_ms, 1_000);
    let mut visual = full_visual();
    let mut mask = default_mask(shape);
    mask.scale = Animatable::Constant {
        value: Vec2 { x: 0.56, y: 0.56 },
    };
    visual.masks.push(mask);
    clip.visual = Some(visual);
    clip
}

#[test]
fn circle_stays_round_while_ellipse_uses_both_axes() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("circle-mask.mp4");
    let mut canonical = project(false);
    let background = solid_clip("itm_circle_bg", color(0, 0, 255), 0, 2_000);
    let circle = masked_clip("itm_circle", MaskShape::Circle, 0);
    let ellipse = masked_clip("itm_ellipse", MaskShape::Ellipse, 1_000);
    canonical.project.sequences[0].tracks.extend([
        track("trk_circle_bg", TrackKind::Video, 0, vec![background]),
        track(
            "trk_circle_shapes",
            TrackKind::Visual,
            1,
            vec![circle, ellipse],
        ),
    ]);

    render(canonical, &BTreeMap::new(), &output);

    assert_red(rgb_at(&output, 0.5, 48, 27));
    assert_red(rgb_at(&output, 1.5, 48, 27));
    assert_blue(rgb_at(&output, 0.5, 68, 27));
    assert_red(rgb_at(&output, 1.5, 68, 27));
    assert_blue(rgb_at(&output, 0.5, 48, 47));
    assert_blue(rgb_at(&output, 1.5, 48, 47));
}

fn assert_red(pixel: [u8; 3]) {
    assert!(pixel[0] > 170 && pixel[2] < 70, "pixel={pixel:?}");
}

fn assert_blue(pixel: [u8; 3]) {
    assert!(
        pixel[2] > 140 && pixel[2] > pixel[0] + 70,
        "pixel={pixel:?}"
    );
}
