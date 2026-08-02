use tempfile::tempdir;

use super::support::*;

#[test]
fn horizontal_and_vertical_shear_move_real_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("shear.mp4");
    let mut value = project(false);
    let mut horizontal = solid_clip("itm_shear_x", color(255, 0, 0), 0, 1_000);
    let mut horizontal_visual = full_visual();
    horizontal_visual.transform.shear = Vec2 { x: 0.75, y: 0.0 };
    horizontal.visual = Some(horizontal_visual);
    let mut vertical = solid_clip("itm_shear_y", color(255, 0, 0), 1_000, 1_000);
    let mut vertical_visual = full_visual();
    vertical_visual.transform.shear = Vec2 { x: 0.0, y: 0.75 };
    vertical.visual = Some(vertical_visual);
    value.project.sequences[0].tracks.push(track(
        "trk_shear",
        TrackKind::Video,
        0,
        vec![horizontal, vertical],
    ));

    let rendered = render(value, &BTreeMap::new(), &output);

    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert_eq!(graph.matches("shear=shx=").count(), 2, "{graph}");
    assert_red(rgb_at(&output, 0.5, 30, 5));
    assert_black(rgb_at(&output, 0.5, 5, 5));
    assert_red(rgb_at(&output, 0.5, 60, 48));
    assert_black(rgb_at(&output, 0.5, 90, 48));
    assert_black(rgb_at(&output, 1.5, 30, 5));
    assert_red(rgb_at(&output, 1.5, 60, 5));
    assert_red(rgb_at(&output, 1.5, 30, 48));
    assert_black(rgb_at(&output, 1.5, 60, 48));
    assert_media_contract(&output, 0, 2.0);
}

fn assert_red(pixel: [u8; 3]) {
    assert!(
        pixel[0] > 180 && pixel[1] < 70 && pixel[2] < 70,
        "expected red, got {pixel:?}"
    );
}

fn assert_black(pixel: [u8; 3]) {
    assert!(
        pixel.iter().all(|value| *value < 40),
        "expected transparent fill over black canvas, got {pixel:?}"
    );
}
