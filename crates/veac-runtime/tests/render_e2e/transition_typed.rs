use tempfile::tempdir;

use super::support::*;

#[path = "transition_typed_observability.rs"]
mod observability;

#[test]
fn every_typed_transition_family_executes_in_real_ffmpeg() {
    let cases = [
        TransitionKind::Fade {
            color: FadeColor::White,
        },
        TransitionKind::Wipe {
            direction: CardinalDirection::Left,
            angle_degrees: 20.0,
            softness: 0.15,
        },
        TransitionKind::Slide {
            direction: CardinalDirection::Up,
            amount: 1.3,
        },
        TransitionKind::Zoom {
            direction: ZoomDirection::In,
            amount: 1.5,
        },
        TransitionKind::Circle {
            direction: CircleDirection::Close,
            softness: 0.1,
        },
        TransitionKind::Pixelize { amount: 0.8 },
    ];
    for (index, kind) in cases.into_iter().enumerate() {
        let pixels = render_transition(kind, 1.0, &format!("family-{index}"));
        assert_eq!(pixels.len(), (WIDTH * HEIGHT * 3) as usize);
    }
}

#[test]
fn typed_direction_shape_and_amount_parameters_change_observable_pixels() {
    let wipe_left = frame(
        TransitionKind::Wipe {
            direction: CardinalDirection::Left,
            angle_degrees: 0.0,
            softness: 0.05,
        },
        1.0,
        "wipe-left",
    );
    let wipe_right = frame(
        TransitionKind::Wipe {
            direction: CardinalDirection::Right,
            angle_degrees: 0.0,
            softness: 0.05,
        },
        1.0,
        "wipe-right",
    );
    assert_opposite("wipe direction", wipe_left, wipe_right);

    let slide_left = frame_at(
        TransitionKind::Slide {
            direction: CardinalDirection::Left,
            amount: 1.0,
        },
        1.0,
        "slide-left",
        WIDTH * 4 / 5,
        HEIGHT / 2,
    );
    let slide_right = frame_at(
        TransitionKind::Slide {
            direction: CardinalDirection::Right,
            amount: 1.0,
        },
        1.0,
        "slide-right",
        WIDTH * 4 / 5,
        HEIGHT / 2,
    );
    assert_opposite("slide direction", slide_right, slide_left);

    let circle_open = frame_at(
        TransitionKind::Circle {
            direction: CircleDirection::Open,
            softness: 0.05,
        },
        1.0,
        "circle-open",
        WIDTH / 2,
        HEIGHT / 2,
    );
    let circle_close = frame_at(
        TransitionKind::Circle {
            direction: CircleDirection::Close,
            softness: 0.05,
        },
        1.0,
        "circle-close",
        WIDTH / 2,
        HEIGHT / 2,
    );
    assert!(u16::from(circle_open[2]) > u16::from(circle_open[0]) + 100);
    assert!(u16::from(circle_close[0]) > u16::from(circle_close[2]) + 100);

    let fast = render_transition(
        TransitionKind::Slide {
            direction: CardinalDirection::Right,
            amount: 0.5,
        },
        0.9,
        "slide-fast",
    );
    let slow = render_transition(
        TransitionKind::Slide {
            direction: CardinalDirection::Right,
            amount: 2.0,
        },
        0.9,
        "slide-slow",
    );
    let fast_blue = blue_pixels(&fast);
    let slow_blue = blue_pixels(&slow);
    assert!(
        fast_blue > slow_blue + (WIDTH * HEIGHT / 8) as usize,
        "slide amount: fast blue={fast_blue}, slow blue={slow_blue}"
    );
}

fn render_transition(kind: TransitionKind, second: f64, name: &str) -> Vec<u8> {
    render_with(kind, name, |output| rgb_frame(output, second))
}

fn render_with<T>(
    kind: TransitionKind,
    name: &str,
    inspect: impl FnOnce(&std::path::Path) -> T,
) -> T {
    let temp = tempdir().unwrap();
    let output = temp.path().join(format!("{name}.mp4"));
    let mut project = project(false);
    let red = visual_solid_clip("itm_typed_red", color(255, 0, 0), 0, 1_250);
    let transition = Transition {
        kind,
        duration: time(500),
        alignment: TransitionAlignment::Centered,
    };
    let blue = visual_solid_clip("itm_typed_blue", color(0, 0, 255), 750, 1_250);
    project.project.sequences[0].tracks.push(track(
        "trk_typed_transition",
        TrackKind::Video,
        0,
        vec![red, blue],
    ));
    add_transition(
        &mut project,
        "seq_main",
        "itm_typed_red",
        "itm_typed_blue",
        transition,
    );
    render(project, &BTreeMap::new(), &output);
    inspect(&output)
}

fn frame(kind: TransitionKind, second: f64, name: &str) -> [u8; 3] {
    frame_at(kind, second, name, WIDTH / 5, HEIGHT / 2)
}

fn frame_at(kind: TransitionKind, second: f64, name: &str, x: u32, y: u32) -> [u8; 3] {
    render_with(kind, name, |output| rgb_at(output, second, x, y))
}

fn blue_pixels(frame: &[u8]) -> usize {
    frame
        .chunks_exact(3)
        .filter(|pixel| u16::from(pixel[2]) > u16::from(pixel[0]) + 100)
        .count()
}

fn assert_opposite(context: &str, left: [u8; 3], right: [u8; 3]) {
    assert!(
        u16::from(left[0]) > u16::from(left[2]) + 100
            && u16::from(right[2]) > u16::from(right[0]) + 100,
        "{context}: left={left:?}, right={right:?}"
    );
}
