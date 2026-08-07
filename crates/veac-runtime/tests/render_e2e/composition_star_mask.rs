use tempfile::tempdir;

use super::support::*;

#[test]
fn star_mask_renders_five_sharp_points_instead_of_radial_lobes() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("star-mask.mp4");
    let mut canonical = project(false);
    let background = solid_clip("itm_star_bg", color(0, 0, 255), 0, 1_000);
    let mut foreground = solid_clip("itm_star", color(255, 0, 0), 0, 1_000);
    let mut visual = full_visual();
    let mut mask = default_mask(MaskShape::Star);
    mask.scale = Animatable::constant(Vec2 { x: 0.72, y: 0.48 });
    mask.rotation_degrees = Animatable::constant(18.0);
    mask.feather_pixels = Animatable::constant(2.0);
    visual.masks.push(mask);
    foreground.visual = Some(visual);
    canonical.project.sequences[0].tracks.extend([
        track("trk_star_bg", TrackKind::Video, 0, vec![background]),
        track("trk_star", TrackKind::Visual, 1, vec![foreground]),
    ]);

    render(canonical, &BTreeMap::new(), &output);
    assert_media_contract(&output, 0, 1.0);
    assert_red("center", rgb_at(&output, 0.5, 48, 27));
    assert_red("spike", rgb_at(&output, 0.5, 63, 30));
    // The old sinusoidal lobe covers this notch; a pentagram must leave it outside.
    assert_blue("notch", rgb_at(&output, 0.5, 58, 25));
    assert_blue("outside", rgb_at(&output, 0.5, 93, 51));
    let edge = rgb_at(&output, 0.5, 77, 33);
    assert!(edge[0] > 35 && edge[2] > 35, "feather edge={edge:?}");
}

fn assert_red(label: &str, pixel: [u8; 3]) {
    assert!(pixel[0] > 170 && pixel[2] < 70, "{label} red={pixel:?}");
}

fn assert_blue(label: &str, pixel: [u8; 3]) {
    assert!(
        pixel[2] > 140 && u16::from(pixel[2]) > u16::from(pixel[0]) + 70,
        "{label} blue={pixel:?}"
    );
}
