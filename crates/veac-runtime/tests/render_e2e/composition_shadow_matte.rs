use tempfile::tempdir;

use super::support::*;

#[test]
fn clip_track_matte_crops_shadow_and_foreground_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("shadow-track-matte.mp4");
    let mut project = project(false);
    let background = solid_clip("itm_shadow_matte_bg", color(0, 0, 0), 0, 1_000);
    let target = shadow_target(1_000, color(255, 255, 255));
    let source = half_canvas_matte();
    project.project.sequences[0].tracks = vec![
        track("trk_shadow_matte_bg", TrackKind::Video, 0, vec![background]),
        track(
            "trk_shadow_matte_target",
            TrackKind::Visual,
            1,
            vec![target],
        ),
        track(
            "trk_shadow_matte_source",
            TrackKind::Visual,
            2,
            vec![source],
        ),
    ];
    add_matte(
        &mut project,
        "seq_main",
        "itm_shadow_matte_source",
        "itm_shadow_matte_target",
        TrackMatteMode::Alpha,
        false,
    );

    render(project, &BTreeMap::new(), &output);
    assert_white(rgb_at(&output, 0.5, 20, 27));
    assert_red(rgb_at(&output, 0.5, 42, 27));
    assert_black(rgb_at(&output, 0.5, 58, 27));
}

#[test]
fn item_apply_processes_shadow_and_foreground_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("shadow-item-apply.mp4");
    let mut project = project(false);
    let background = solid_clip("itm_shadow_apply_bg", color(0, 0, 0), 0, 2_000);
    let target = shadow_target(2_000, color(60, 60, 60));
    let sequence = &mut project.project.sequences[0];
    sequence.tracks = vec![
        track("trk_shadow_apply_bg", TrackKind::Video, 0, vec![background]),
        track(
            "trk_shadow_apply_target",
            TrackKind::Visual,
            1,
            vec![target],
        ),
    ];
    sequence.applies.push(apply(
        "apl_shadow_item",
        ApplyTarget::ItemSet {
            item_ids: vec![ItemId::new("itm_shadow_matte_target").unwrap()],
        },
        500,
        1_000,
        vec![brightness_stage("aps_shadow_item", 0.25)],
    ));

    render(project, &BTreeMap::new(), &output);
    assert_changed(rgb_at(&output, 0.25, 20, 27), rgb_at(&output, 1.0, 20, 27));
    assert_changed(rgb_at(&output, 0.25, 50, 27), rgb_at(&output, 1.0, 50, 27));
}

fn shadow_target(duration_ms: i64, fill: Color) -> Clip {
    let mut target = solid_clip("itm_shadow_matte_target", fill, 0, duration_ms);
    let mut visual = framed_visual(Anchor::TopLeft, None);
    visual.transform.anchor = Vec2 { x: 0.0, y: 0.0 };
    visual.transform.scale = Animatable::constant(Vec2 {
        x: 24.0 / f64::from(WIDTH),
        y: 14.0 / f64::from(HEIGHT),
    });
    visual.placement = Placement::Absolute {
        position: Point {
            x: pixels(8.0),
            y: pixels(20.0),
        },
    };
    visual.card = Some(CardStyle {
        corner_radius_pixels: 0.0,
        shadow: Some(Shadow {
            color: color(255, 0, 0),
            offset: Vec2 { x: 30.0, y: 0.0 },
            blur_pixels: 0.0,
            opacity: 1.0,
        }),
    });
    target.visual = Some(visual);
    target
}

fn half_canvas_matte() -> Clip {
    let mut source = solid_clip("itm_shadow_matte_source", color(255, 255, 255), 0, 1_000);
    let mut visual = full_visual();
    let mut mask = default_mask(MaskShape::Rectangle);
    mask.position = Animatable::constant(Vec2 { x: 0.25, y: 0.5 });
    mask.scale = Animatable::constant(Vec2 { x: 0.5, y: 1.0 });
    visual.masks.push(mask);
    source.visual = Some(visual);
    source
}

fn assert_white(pixel: [u8; 3]) {
    assert!(pixel.iter().all(|channel| *channel > 200), "{pixel:?}");
}

fn assert_red(pixel: [u8; 3]) {
    assert!(
        pixel[0] > 170 && pixel[1] < 70 && pixel[2] < 70,
        "{pixel:?}"
    );
}

fn assert_black(pixel: [u8; 3]) {
    assert!(pixel.iter().all(|channel| *channel < 30), "{pixel:?}");
}

fn assert_changed(before: [u8; 3], during: [u8; 3]) {
    let difference: u16 = before
        .into_iter()
        .zip(during)
        .map(|(left, right)| u16::from(left.abs_diff(right)))
        .sum();
    assert!(difference > 45, "before={before:?}, during={during:?}");
}
