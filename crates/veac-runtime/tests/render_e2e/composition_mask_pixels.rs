use tempfile::tempdir;

use super::support::*;

#[test]
fn rectangle_and_ellipse_feather_use_equal_pixel_widths_on_both_axes() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("mask-pixel-feather.mp4");
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_feather_bg",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_feather_bg", color(0, 0, 0), 0, 2_000)],
        ),
        track(
            "trk_feather_fg",
            TrackKind::Visual,
            1,
            vec![
                masked_white(
                    "itm_rectangle_feather",
                    MaskShape::Rectangle,
                    0,
                    1_000,
                    6.0,
                    0.0,
                ),
                masked_white(
                    "itm_ellipse_feather",
                    MaskShape::Ellipse,
                    1_000,
                    1_000,
                    6.0,
                    0.0,
                ),
            ],
        ),
    ]);

    render(canonical, &BTreeMap::new(), &output);
    for seconds in [0.5, 1.5] {
        assert_equal_axis_feather(&rgb_frame(&output, seconds));
    }
}

#[test]
fn rectangle_expansion_moves_each_non_uniform_axis_by_the_same_pixels() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("mask-pixel-expansion.mp4");
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.extend([
        track(
            "trk_expansion_bg",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_expansion_bg", color(0, 0, 0), 0, 2_000)],
        ),
        track(
            "trk_expansion_fg",
            TrackKind::Visual,
            1,
            vec![
                masked_white(
                    "itm_expansion_zero",
                    MaskShape::Rectangle,
                    0,
                    1_000,
                    4.0,
                    0.0,
                ),
                masked_white(
                    "itm_expansion_four",
                    MaskShape::Rectangle,
                    1_000,
                    1_000,
                    4.0,
                    4.0,
                ),
            ],
        ),
    ]);

    render(canonical, &BTreeMap::new(), &output);
    let base = rgb_frame(&output, 0.5);
    let expanded = rgb_frame(&output, 1.5);
    let base_right = half_coverage((48..96).map(|x| (x, gray(&base, x, 27))));
    let expanded_right = half_coverage((48..96).map(|x| (x, gray(&expanded, x, 27))));
    let base_top = half_coverage((0..=27).map(|y| (y, gray(&base, 48, y))));
    let expanded_top = half_coverage((0..=27).map(|y| (y, gray(&expanded, 48, y))));
    assert!(base_right.abs_diff(78) <= 1, "base right={base_right}");
    assert!(base_top.abs_diff(18) <= 1, "base top={base_top}");
    let horizontal = expanded_right as i32 - base_right as i32;
    let vertical = base_top as i32 - expanded_top as i32;
    assert!(
        (3..=5).contains(&horizontal),
        "right {base_right}->{expanded_right}"
    );
    assert!(
        (3..=5).contains(&vertical),
        "top {base_top}->{expanded_top}"
    );
    assert!(
        horizontal.abs_diff(vertical) <= 1,
        "horizontal={horizontal} vertical={vertical}"
    );
}

fn masked_white(
    id: &str,
    shape: MaskShape,
    start_ms: i64,
    duration_ms: i64,
    feather: f64,
    expansion: f64,
) -> Clip {
    let mut clip = solid_clip(id, color(255, 255, 255), start_ms, duration_ms);
    let mut visual = full_visual();
    let mut mask = default_mask(shape);
    mask.scale = Animatable::constant(Vec2 {
        x: 0.625,
        y: 1.0 / 3.0,
    });
    mask.feather_pixels = Animatable::constant(feather);
    mask.expansion_pixels = Animatable::constant(expansion);
    visual.masks.push(mask);
    clip.visual = Some(visual);
    clip
}

fn assert_equal_axis_feather(frame: &[u8]) {
    let horizontal = [72, 75, 78, 81, 84].map(|x| gray(frame, x, 27));
    let vertical = [24, 21, 18, 15, 12].map(|y| gray(frame, 48, y));
    assert_profile(&horizontal);
    assert_profile(&vertical);
    for (x, y) in horizontal.into_iter().zip(vertical) {
        assert!(
            x.abs_diff(y) <= 18,
            "horizontal={horizontal:?} vertical={vertical:?}"
        );
    }
    let horizontal_width = mixed_pixels((48..96).map(|x| gray(frame, x, 27)));
    let vertical_width = mixed_pixels((0..=27).map(|y| gray(frame, 48, y)));
    assert!(
        (9..=13).contains(&horizontal_width),
        "horizontal={horizontal_width}"
    );
    assert!(
        (9..=13).contains(&vertical_width),
        "vertical={vertical_width}"
    );
    assert!(horizontal_width.abs_diff(vertical_width) <= 1);
}

fn gray(frame: &[u8], x: u32, y: u32) -> u8 {
    let index = ((y * WIDTH + x) * 3) as usize;
    let sum = frame[index..index + 3]
        .iter()
        .map(|value| u16::from(*value))
        .sum::<u16>();
    (sum / 3) as u8
}

fn assert_profile(values: &[u8; 5]) {
    assert!(values[0] >= 225 && values[4] <= 25, "profile={values:?}");
    assert!((90..=165).contains(&values[2]), "profile={values:?}");
    assert!(
        values.windows(2).all(|pair| pair[0] > pair[1]),
        "profile={values:?}"
    );
}

fn mixed_pixels(values: impl Iterator<Item = u8>) -> usize {
    values.filter(|value| (24..=231).contains(value)).count()
}

fn half_coverage(values: impl Iterator<Item = (u32, u8)>) -> u32 {
    values
        .min_by_key(|(_, value)| value.abs_diff(128))
        .expect("non-empty scan line")
        .0
}
