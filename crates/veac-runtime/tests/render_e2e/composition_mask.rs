use tempfile::tempdir;

use super::support::*;

#[test]
fn arbitrary_path_mask_position_curve_moves_real_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("path-mask.mp4");
    let mut project = project(false);
    let background = solid_clip("itm_path_bg", color(0, 0, 255), 0, 2_000);
    let mut foreground = solid_clip("itm_path_fg", color(255, 0, 0), 0, 2_000);
    let mut visual = full_visual();
    let mut mask = default_mask(MaskShape::Path {
        points: vec![
            Vec2 { x: 0.15, y: 0.25 },
            Vec2 { x: 0.85, y: 0.20 },
            Vec2 { x: 0.70, y: 0.80 },
            Vec2 { x: 0.25, y: 0.65 },
        ],
    });
    mask.position = vec_curve("path_position", (0.30, 0.50), (0.70, 0.50));
    mask.scale = vec_curve("path_scale", (0.55, 0.55), (0.65, 0.50));
    visual.masks.push(mask);
    foreground.visual = Some(visual);
    project.project.sequences[0].tracks.extend([
        track("trk_path_bg", TrackKind::Video, 0, vec![background]),
        track("trk_path_fg", TrackKind::Visual, 1, vec![foreground]),
    ]);

    render(project, &BTreeMap::new(), &output);
    assert_red(rgb_at(&output, 0.15, 29, 27));
    assert_blue(rgb_at(&output, 0.15, 67, 27));
    assert_blue(rgb_at(&output, 1.85, 29, 27));
    assert_red(rgb_at(&output, 1.85, 67, 27));
}

#[test]
fn mask_rotation_scale_feather_and_expansion_curves_are_frame_evaluated() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("mask-curves.mp4");
    let mut project = project(false);
    let background = solid_clip("itm_curve_bg", color(0, 0, 255), 0, 2_000);
    let mut foreground = solid_clip("itm_curve_fg", color(255, 0, 0), 0, 2_000);
    let mut visual = full_visual();
    let mut mask = default_mask(MaskShape::Rectangle);
    mask.scale = vec_curve("mask_scale", (0.60, 0.20), (0.50, 0.30));
    mask.rotation_degrees = number_curve("mask_rotation", 0.0, 90.0);
    mask.feather_pixels = number_curve("mask_feather", 0.0, 3.0);
    mask.expansion_pixels = number_curve("mask_expansion", 0.0, 4.0);
    visual.masks.push(mask);
    foreground.visual = Some(visual);
    project.project.sequences[0].tracks.extend([
        track("trk_curve_bg", TrackKind::Video, 0, vec![background]),
        track("trk_curve_fg", TrackKind::Visual, 1, vec![foreground]),
    ]);

    render(project, &BTreeMap::new(), &output);
    assert_red(rgb_at(&output, 0.05, 70, 27));
    assert_blue(rgb_at(&output, 0.05, 48, 42));
    assert_blue(rgb_at(&output, 1.85, 70, 27));
    assert_red(rgb_at(&output, 1.85, 48, 42));
    let feathered = rgb_at(&output, 1.85, 48, 45);
    assert!(feathered[0] > 45 && feathered[2] > 45, "edge={feathered:?}");
}

fn vec_curve(id: &str, start: (f64, f64), end: (f64, f64)) -> Animatable<Vec2> {
    Animatable::Keyframes {
        keyframes: vec![
            vec_key(&format!("kf_{id}_start"), 0, start),
            vec_key(&format!("kf_{id}_end"), 2_000, end),
        ],
    }
}

fn vec_key(id: &str, at: i64, value: (f64, f64)) -> Keyframe<Vec2> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value: Vec2 {
            x: value.0,
            y: value.1,
        },
        interpolation: Interpolation::Linear,
    }
}

fn number_curve(id: &str, start: f64, end: f64) -> Animatable<f64> {
    Animatable::Keyframes {
        keyframes: vec![
            number_key(&format!("kf_{id}_start"), 0, start),
            number_key(&format!("kf_{id}_end"), 2_000, end),
        ],
    }
}

fn number_key(id: &str, at: i64, value: f64) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    }
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
