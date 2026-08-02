use super::support::*;

#[path = "layout_matrix/model.rs"]
mod model;
use model::*;

#[test]
fn all_nine_anchors_place_a_real_overlay_at_the_expected_edge_or_center() {
    let cases = [
        (Anchor::TopLeft, 0, 0),
        (Anchor::Top, 44, 0),
        (Anchor::TopRight, 88, 0),
        (Anchor::Left, 0, 24),
        (Anchor::Center, 44, 24),
        (Anchor::Right, 88, 24),
        (Anchor::BottomLeft, 0, 48),
        (Anchor::Bottom, 44, 48),
        (Anchor::BottomRight, 88, 48),
    ];
    let mut value = project(false);
    let base = solid_clip("itm_base", color(0, 0, 0), 0, 1_800);
    let overlays = cases
        .iter()
        .enumerate()
        .map(|(index, (anchor, _, _))| {
            layout_clip(
                &format!("itm_anchor_{index}"),
                index as i64 * 200,
                200,
                Placement::Anchor {
                    anchor: *anchor,
                    inset: Vec2 { x: 0.0, y: 0.0 },
                },
                FitMode::Fill,
            )
        })
        .collect();
    value.project.sequences[0].tracks = vec![
        track("trk_base", TrackKind::Video, 0, vec![base]),
        track("trk_overlay", TrackKind::Visual, 1, overlays),
    ];
    let (_temp, output) = render_layout(&value);

    for (index, (_, x, y)) in cases.into_iter().enumerate() {
        let frame = rgb_frame(&output, index as f64 * 0.2 + 0.1);
        assert_red(pixel_at(&frame, WIDTH, x + 4, y + 3));
        let outside = if x == 0 && y == 0 { (95, 53) } else { (0, 0) };
        assert_black(pixel_at(&frame, WIDTH, outside.0, outside.1));
    }
}

#[test]
fn pixel_percent_and_normalized_absolute_positions_render_observably() {
    let cases = [
        (
            length(7.0, LengthUnit::Pixels),
            length(9.0, LengthUnit::Pixels),
            8,
            8,
        ),
        (
            length(25.0, LengthUnit::Percent),
            length(50.0, LengthUnit::Percent),
            28,
            30,
        ),
        (
            length(0.5, LengthUnit::Normalized),
            length(0.25, LengthUnit::Normalized),
            52,
            17,
        ),
    ];
    let mut value = project(false);
    let base = solid_clip("itm_base", color(0, 0, 0), 0, 1_800);
    let overlays = cases
        .iter()
        .enumerate()
        .map(|(index, (x, y, _, _))| {
            layout_clip(
                &format!("itm_absolute_{index}"),
                index as i64 * 600,
                600,
                Placement::Absolute {
                    position: Point { x: *x, y: *y },
                },
                FitMode::Fill,
            )
        })
        .collect();
    value.project.sequences[0].tracks = vec![
        track("trk_base", TrackKind::Video, 0, vec![base]),
        track("trk_overlay", TrackKind::Visual, 1, overlays),
    ];
    let (_temp, output) = render_layout(&value);

    for (index, (_, _, x, y)) in cases.into_iter().enumerate() {
        let frame = rgb_frame(&output, index as f64 * 0.6 + 0.3);
        assert_red(pixel_at(&frame, WIDTH, x, y));
        assert_black(pixel_at(&frame, WIDTH, 95, 53));
    }
}

#[test]
fn contain_cover_and_fill_have_distinct_real_pixel_geometry() {
    let temp = tempfile::tempdir().unwrap();
    let source = layout_video_fixture(temp.path());
    let mut value = project(false);
    value.project.materials.push(material(
        "med_layout",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    let base = solid_clip("itm_base", color(0, 0, 0), 0, 1_800);
    let overlays = [FitMode::Contain, FitMode::Cover, FitMode::Fill]
        .into_iter()
        .enumerate()
        .map(|(index, fit)| media_layout_clip(&format!("itm_fit_{index}"), index as i64 * 600, fit))
        .collect();
    value.project.sequences[0].tracks = vec![
        track("trk_base", TrackKind::Video, 0, vec![base]),
        track("trk_overlay", TrackKind::Visual, 1, overlays),
    ];
    let output = temp.path().join("fit-modes.mp4");
    let assets = BTreeMap::from([("med_layout".to_owned(), source)]);
    render(value, &assets, &output);
    let contain = rgb_frame(&output, 0.3);
    let cover = rgb_frame(&output, 0.9);
    let fill = rgb_frame(&output, 1.5);

    assert_black(pixel_at(&contain, WIDTH, 40, 12));
    assert_colored(pixel_at(&contain, WIDTH, 40, 27));
    assert_colored(pixel_at(&cover, WIDTH, 40, 12));
    let cover_left = pixel_at(&cover, WIDTH, 33, 19);
    let fill_left = pixel_at(&fill, WIDTH, 33, 19);
    let difference: u64 = cover
        .iter()
        .zip(&fill)
        .map(|(left, right)| u64::from(left.abs_diff(*right)))
        .sum();
    assert!(difference > 20_000, "difference={difference}");
    assert_ne!(cover_left, fill_left);
}
