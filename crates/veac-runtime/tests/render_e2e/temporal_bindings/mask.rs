use tempfile::tempdir;

use super::super::support::*;
use super::{install, progress_curve};

#[test]
fn progress_bindings_move_and_scale_a_clip_mask_in_rendered_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-mask-position-scale.mp4");
    let mut project = project(false);
    let position = install(
        &mut project,
        "mask_position",
        "itm_temporal_mask",
        TemporalType::Vec2,
        progress_curve(vector(0.25, 0.5), vector(0.75, 0.5)),
        1,
    );
    let scale = install(
        &mut project,
        "mask_scale",
        "itm_temporal_mask",
        TemporalType::Vec2,
        progress_curve(vector(0.18, 0.24), vector(0.45, 0.5)),
        1,
    );
    let mut mask = default_mask(MaskShape::Circle);
    mask.position = Animatable::Binding {
        binding_id: position,
    };
    mask.scale = Animatable::Binding { binding_id: scale };
    add_scene(&mut project, mask);
    render(project, &BTreeMap::new(), &output);

    let early = frame_stats(&rgb_frame(&output, 0.1));
    let late = frame_stats(&rgb_frame(&output, 0.8));
    assert!(
        late.centroid_x > early.centroid_x + 25.0,
        "{early:?} -> {late:?}"
    );
    assert!(late.ratio > early.ratio * 2.2, "{early:?} -> {late:?}");
}

#[test]
fn progress_angle_binding_rotates_a_non_square_mask() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-mask-rotation.mp4");
    let mut project = project(false);
    let rotation = install(
        &mut project,
        "mask_rotation",
        "itm_temporal_mask",
        TemporalType::Angle,
        progress_curve(angle(0.0), angle(90.0)),
        1,
    );
    let mut mask = default_mask(MaskShape::Rectangle);
    mask.scale = Animatable::constant(Vec2 { x: 0.55, y: 0.2 });
    mask.rotation_degrees = Animatable::Binding {
        binding_id: rotation,
    };
    add_scene(&mut project, mask);
    render(project, &BTreeMap::new(), &output);

    let early = lit_bounds(&rgb_frame(&output, 0.05));
    let late = lit_bounds(&rgb_frame(&output, 0.85));
    assert!(early.0 > early.1 * 2, "early bounds {early:?}");
    assert!(late.1 > late.0 * 2, "late bounds {late:?}");
}

#[test]
fn progress_bindings_drive_mask_feather_and_expansion() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("temporal-mask-edge.mp4");
    let mut project = project(false);
    let expansion = install(
        &mut project,
        "mask_expansion",
        "itm_temporal_mask",
        TemporalType::Scalar,
        progress_curve(scalar(0.0), scalar(7.0)),
        1,
    );
    let feather = install(
        &mut project,
        "mask_feather",
        "itm_temporal_mask",
        TemporalType::Scalar,
        progress_curve(scalar(0.5), scalar(7.0)),
        1,
    );
    let mut mask = default_mask(MaskShape::Circle);
    mask.scale = Animatable::constant(Vec2 { x: 0.3, y: 0.54 });
    mask.expansion_pixels = Animatable::Binding {
        binding_id: expansion,
    };
    mask.feather_pixels = Animatable::Binding {
        binding_id: feather,
    };
    add_scene(&mut project, mask);
    render(project, &BTreeMap::new(), &output);

    let early = rgb_frame(&output, 0.1);
    let late = rgb_frame(&output, 0.8);
    assert!(lit_pixels(&late) > lit_pixels(&early) * 13 / 10);
    assert!(soft_pixels(&late) > soft_pixels(&early) * 2);
}

fn add_scene(project: &mut ProjectEnvelope, mask: Mask) {
    let mut foreground = solid_clip("itm_temporal_mask", color(250, 250, 250), 0, 1_000);
    let mut visual = full_visual();
    visual.masks.push(mask);
    foreground.visual = Some(visual);
    project.project.sequences[0].tracks.extend([
        track(
            "trk_temporal_mask_base",
            TrackKind::Video,
            0,
            vec![solid_clip(
                "itm_temporal_mask_base",
                color(0, 0, 0),
                0,
                1_000,
            )],
        ),
        track("trk_temporal_mask", TrackKind::Visual, 1, vec![foreground]),
    ]);
}

fn scalar(value: f64) -> TemporalValue {
    TemporalValue::Scalar { value }
}

fn angle(degrees: f64) -> TemporalValue {
    TemporalValue::Angle { degrees }
}

fn vector(x: f64, y: f64) -> TemporalValue {
    TemporalValue::Vec2 {
        value: Vec2 { x, y },
    }
}

fn lit_pixels(frame: &[u8]) -> usize {
    frame.chunks_exact(3).filter(|pixel| pixel[0] > 12).count()
}

fn soft_pixels(frame: &[u8]) -> usize {
    frame
        .chunks_exact(3)
        .filter(|pixel| (20..=230).contains(&pixel[0]))
        .count()
}
