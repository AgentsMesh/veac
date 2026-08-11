use super::support::*;
use tempfile::tempdir;

#[test]
fn opaque_media_rotation_keeps_padded_pixels_transparent() {
    let temp = tempdir().unwrap();
    let source = color_video_fixture(temp.path(), "opaque-red", "0xEC232A");
    let base = solid_clip("itm_rotation_base", color(24, 56, 79), 0, 1_000);
    let mut layer = media_clip("itm_rotation_layer", "med_opaque_red", 0, 1_000);
    let mut visual = framed_visual(Anchor::Center, None);
    visual.transform.anchor = Vec2 { x: 0.25, y: 0.5 };
    visual.transform.scale = Animatable::constant(Vec2 { x: 0.72, y: 0.72 });
    visual.transform.rotation_degrees = Animatable::constant(8.0);
    layer.visual = Some(visual);

    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_opaque_red",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks = vec![
        track("trk_rotation_base", TrackKind::Video, 0, vec![base]),
        track("trk_rotation_layer", TrackKind::Visual, 1, vec![layer]),
    ];
    let output = temp.path().join("opaque-media-rotation.mp4");
    let assets = BTreeMap::from([("med_opaque_red".to_owned(), source)]);
    render(canonical, &assets, &output);

    for (x, y) in [(4, 4), (4, 49), (90, 4)] {
        assert_navy(rgb_at(&output, 0.5, x, y));
    }
    assert_red(rgb_at(&output, 0.5, 60, 27));
}

#[test]
fn centered_opaque_media_rotation_keeps_corners_transparent() {
    let temp = tempdir().unwrap();
    let source = color_video_fixture(temp.path(), "centered-red", "0xEC232A");
    let base = solid_clip("itm_centered_base", color(24, 56, 79), 0, 1_000);
    let mut layer = media_clip("itm_centered_layer", "med_centered_red", 0, 1_000);
    let mut visual = framed_visual(Anchor::Center, None);
    visual.transform.scale = Animatable::constant(Vec2 { x: 0.72, y: 0.72 });
    visual.transform.rotation_degrees = Animatable::constant(8.0);
    layer.visual = Some(visual);

    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_centered_red",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks = vec![
        track("trk_centered_base", TrackKind::Video, 0, vec![base]),
        track("trk_centered_layer", TrackKind::Visual, 1, vec![layer]),
    ];
    let output = temp.path().join("centered-media-rotation.mp4");
    let assets = BTreeMap::from([("med_centered_red".to_owned(), source)]);
    render(canonical, &assets, &output);

    for (x, y) in [(4, 4), (4, 49), (90, 4)] {
        assert_navy(rgb_at(&output, 0.5, x, y));
    }
    assert_red(rgb_at(&output, 0.5, 48, 27));
}

#[test]
fn opaque_media_contain_fit_keeps_letterbox_pixels_transparent() {
    let temp = tempdir().unwrap();
    let source = color_video_fixture(temp.path(), "contained-red", "0xEC232A");
    let base = solid_clip("itm_contain_base", color(24, 56, 79), 0, 1_000);
    let mut layer = media_clip("itm_contain_layer", "med_contained_red", 0, 1_000);
    let mut visual = framed_visual(Anchor::Center, Some((50.0, 50.0)));
    visual.frame.as_mut().unwrap().fit = FitMode::Contain;
    layer.visual = Some(visual);

    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_contained_red",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks = vec![
        track("trk_contain_base", TrackKind::Video, 0, vec![base]),
        track("trk_contain_layer", TrackKind::Visual, 1, vec![layer]),
    ];
    let output = temp.path().join("opaque-media-contain.mp4");
    let assets = BTreeMap::from([("med_contained_red".to_owned(), source)]);
    render(canonical, &assets, &output);

    assert_navy(rgb_at(&output, 0.5, 48, 7));
    assert_navy(rgb_at(&output, 0.5, 48, 47));
    assert_red(rgb_at(&output, 0.5, 48, 27));
}

fn assert_navy(pixel: [u8; 3]) {
    assert!(pixel[0] < 45 && pixel[1] < 80 && pixel[2] > 55, "{pixel:?}");
}

fn assert_red(pixel: [u8; 3]) {
    assert!(
        pixel[0] > 180 && pixel[1] < 80 && pixel[2] < 90,
        "{pixel:?}"
    );
}
