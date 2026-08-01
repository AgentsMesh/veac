use std::{path::Path, process::Command};

use super::support::*;
use tempfile::tempdir;

#[test]
fn animated_position_uses_a_time_aware_compositor() {
    let temp = tempdir().unwrap();
    let source = color_video_fixture(temp.path(), "move-red", "0xEC232A");
    let base = solid_clip("itm_move_base", color(14, 38, 72), 0, 2_000);
    let mut layer = media_clip("itm_move_layer", "med_move_red", 0, 2_000);
    let mut visual = framed_visual(Anchor::Center, Some((10.0, 10.0)));
    visual.transform.position = Animatable::Keyframes {
        keyframes: vec![
            point_key("kf_move_left", 0, -30.0),
            point_key("kf_move_right", 2_000, 30.0),
        ],
    };
    layer.visual = Some(visual);
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_move_red",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks = vec![
        track("trk_move_base", TrackKind::Video, 0, vec![base]),
        track("trk_move_layer", TrackKind::Visual, 1, vec![layer]),
    ];
    let output = temp.path().join("animated-position.mp4");
    let assets = BTreeMap::from([("med_move_red".to_owned(), source)]);
    render(canonical, &assets, &output);

    assert_red(rgb_at(&output, 0.25, 25, 27));
    assert_navy(rgb_at(&output, 0.25, 70, 27));
    assert_red(rgb_at(&output, 1.75, 70, 27));
    assert_navy(rgb_at(&output, 1.75, 25, 27));
}

#[test]
fn non_square_media_at_its_left_boundary_is_fully_clipped() {
    let temp = tempdir().unwrap();
    let source = hd_red_fixture(temp.path());
    let mut layer = media_clip("itm_hd", "med_hd", 0, 500);
    let mut visual = framed_visual(Anchor::TopLeft, None);
    visual.placement = Placement::Absolute {
        position: Point {
            x: pixels(-1_920.0),
            y: pixels(0.0),
        },
    };
    layer.visual = Some(visual);
    let mut canonical = project(false);
    canonical.project.materials.push(material(
        "med_hd",
        MaterialKind::Video,
        StreamChoice::Auto,
        StreamChoice::Disabled,
    ));
    canonical.project.sequences[0].tracks = vec![
        track(
            "trk_hd_base",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_hd_base", color(14, 38, 72), 0, 500)],
        ),
        track("trk_hd", TrackKind::Visual, 1, vec![layer]),
    ];
    let output = temp.path().join("non-square-offscreen.mp4");
    let assets = BTreeMap::from([("med_hd".to_owned(), source)]);
    render(canonical, &assets, &output);

    for (x, y) in [(4, 4), (48, 27), (90, 48)] {
        assert_navy(rgb_at(&output, 0.25, x, y));
    }
}

#[test]
fn generated_source_honors_an_authored_frame_and_center_placement() {
    let temp = tempdir().unwrap();
    let mut layer = solid_clip("itm_generated", color(236, 35, 42), 0, 500);
    layer.visual = Some(framed_visual(Anchor::Center, Some((10.0, 10.0))));
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks = vec![
        track(
            "trk_generated_base",
            TrackKind::Video,
            0,
            vec![solid_clip("itm_generated_base", color(14, 38, 72), 0, 500)],
        ),
        track("trk_generated", TrackKind::Visual, 1, vec![layer]),
    ];
    let output = temp.path().join("generated-frame.mp4");
    render(canonical, &BTreeMap::new(), &output);

    for (x, y) in [(45, 24), (48, 27), (50, 29)] {
        assert_red(rgb_at(&output, 0.25, x, y));
    }
    for (x, y) in [(40, 19), (55, 34), (4, 4), (90, 48)] {
        assert_navy(rgb_at(&output, 0.25, x, y));
    }
}

fn point_key(id: &str, milliseconds: i64, x: f64) -> Keyframe<Point> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(milliseconds),
        value: Point {
            x: pixels(x),
            y: pixels(0.0),
        },
        interpolation: Interpolation::Linear,
    }
}

fn hd_red_fixture(directory: &Path) -> PathBuf {
    let output = directory.join("hd-red.mp4");
    let result = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=1920x1080:r=10:d=0.5",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-y",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(result.status.success());
    output
}

fn assert_red(pixel: [u8; 3]) {
    assert!(
        pixel[0] > 180 && pixel[1] < 75 && pixel[2] < 85,
        "{pixel:?}"
    );
}

fn assert_navy(pixel: [u8; 3]) {
    assert!(pixel[0] < 35 && pixel[1] < 65 && pixel[2] > 50, "{pixel:?}");
}
