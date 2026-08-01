use super::support::*;
use tempfile::tempdir;

#[test]
fn static_position_tracks_animated_scale_dimensions() {
    let temp = tempdir().unwrap();
    let source = color_video_fixture(temp.path(), "scale-red", "0xEC232A");
    let base = solid_clip("itm_scale_base", color(14, 38, 72), 0, 2_000);
    let mut layer = media_clip("itm_scale_layer", "med_scale_red", 0, 2_000);
    let mut visual = framed_visual(Anchor::Center, Some((10.0, 10.0)));
    visual.transform.scale = Animatable::Keyframes {
        keyframes: vec![
            scale_key("kf_scale_10", 0, 1.0),
            scale_key("kf_scale_15", 500, 1.5),
            scale_key("kf_scale_20", 1_000, 2.0),
            scale_key("kf_scale_25", 1_500, 2.5),
        ],
    };
    layer.visual = Some(visual);
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_scale_red",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks = vec![
        track("trk_scale_base", TrackKind::Video, 0, vec![base]),
        track("trk_scale_layer", TrackKind::Visual, 1, vec![layer]),
    ];
    let output = temp.path().join("animated-scale-placement.mp4");
    let assets = BTreeMap::from([("med_scale_red".to_owned(), source)]);
    render(canonical, &assets, &output);
    for (second, expected_left, expected_width) in [
        (0.25, 43, 10),
        (0.75, 40, 15),
        (1.25, 38, 20),
        (1.75, 35, 25),
    ] {
        let frame = rgb_frame(&output, second);
        let bounds = red_bounds(96, &frame);
        assert!(bounds.0.abs_diff(expected_left) <= 2, "{bounds:?}");
        assert!(bounds.2.abs_diff(expected_width) <= 2, "{bounds:?}");
        assert!((bounds.0 + bounds.1).abs_diff(95) <= 3, "{bounds:?}");
    }
}

#[test]
fn fully_off_canvas_offsets_remain_transparent() {
    let temp = tempdir().unwrap();
    let base = solid_clip("itm_offset_base", color(14, 38, 72), 0, 2_000);
    let left = offset_layer("itm_far_left", 0, -150.0);
    let right = offset_layer("itm_far_right", 1_000, 150.0);
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks = vec![
        track("trk_offset_base", TrackKind::Video, 0, vec![base]),
        track("trk_offset_layers", TrackKind::Visual, 1, vec![left, right]),
    ];
    let output = temp.path().join("off-canvas-placement.mp4");
    render(canonical, &BTreeMap::new(), &output);

    for second in [0.5, 1.5] {
        for (x, y) in [(4, 4), (48, 27), (90, 48)] {
            assert_navy(rgb_at(&output, second, x, y));
        }
    }
}

fn offset_layer(id: &str, start: i64, x: f64) -> Clip {
    let mut layer = solid_clip(id, color(236, 35, 42), start, 1_000);
    let mut visual = framed_visual(Anchor::Center, Some((20.0, 20.0)));
    visual.placement = Placement::Absolute {
        position: Point {
            x: pixels(x),
            y: pixels(27.0),
        },
    };
    layer.visual = Some(visual);
    layer
}

fn scale_key(id: &str, milliseconds: i64, value: f64) -> Keyframe<Vec2> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(milliseconds),
        value: Vec2 { x: value, y: value },
        interpolation: Interpolation::Hold,
    }
}

fn red_bounds(width: u32, pixels: &[u8]) -> (u32, u32, u32) {
    let mut left = width;
    let mut right = 0;
    let mut count = 0_u32;
    for (index, pixel) in pixels.chunks_exact(3).enumerate() {
        if pixel[0] > 150
            && u16::from(pixel[0]) > u16::from(pixel[1]) + 80
            && u16::from(pixel[0]) > u16::from(pixel[2]) + 70
        {
            let x = index as u32 % width;
            left = left.min(x);
            right = right.max(x);
            count += 1;
        }
    }
    assert!(count > 0);
    (left, right, right - left + 1)
}

fn assert_navy(pixel: [u8; 3]) {
    assert!(pixel[0] < 35 && pixel[1] < 65 && pixel[2] > 50, "{pixel:?}");
}
