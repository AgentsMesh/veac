use tempfile::tempdir;

use crate::support::*;

#[test]
fn fade_colors_have_distinct_observable_midpoints() {
    let transparent = patterned_frame(TransitionKind::Fade {
        color: FadeColor::Transparent,
    });
    let black = patterned_frame(TransitionKind::Fade {
        color: FadeColor::Black,
    });
    let white = patterned_frame(TransitionKind::Fade {
        color: FadeColor::White,
    });

    let transparent_mean = mean_channel(&transparent);
    assert!(mean_channel(&black) + 35.0 < transparent_mean);
    assert!(transparent_mean + 35.0 < mean_channel(&white));
}

#[test]
fn zoom_out_changes_spatial_sampling() {
    let zoom_out = patterned_frame(TransitionKind::Zoom {
        direction: ZoomDirection::Out,
        amount: 0.8,
    });
    let zoom_in = patterned_frame(TransitionKind::Zoom {
        direction: ZoomDirection::In,
        amount: 0.8,
    });

    assert!(changed_channels(&zoom_out, &zoom_in, 12) > 1_000);
}

#[test]
fn pixelize_amount_changes_the_rendered_block_size() {
    let subtle = patterned_frame(TransitionKind::Pixelize { amount: 0.05 });
    let coarse = patterned_frame(TransitionKind::Pixelize { amount: 1.0 });

    assert!(changed_channels(&subtle, &coarse, 12) > 1_000);
    let subtle_changes = horizontal_changes(&subtle, 4);
    let coarse_changes = horizontal_changes(&coarse, 4);
    assert!(
        subtle_changes > coarse_changes * 2,
        "expected finer blocks at low amount: subtle={subtle_changes}, coarse={coarse_changes}"
    );
}

#[test]
fn wipe_angle_and_softness_change_the_rendered_boundary() {
    let horizontal = patterned_frame(wipe(0.0, 0.0));
    let angled = patterned_frame(wipe(90.0, 0.0));
    let soft = patterned_frame(wipe(0.0, 0.8));

    assert!(changed_channels(&horizontal, &angled, 12) > 1_000);
    assert!(changed_channels(&horizontal, &soft, 12) > 300);
}

#[test]
fn circle_softness_changes_the_rendered_boundary() {
    let hard = patterned_frame(circle(0.0));
    let soft = patterned_frame(circle(0.8));

    assert!(changed_channels(&hard, &soft, 12) > 300);
}

#[test]
fn custom_transition_progresses_from_outgoing_to_incoming() {
    super::render_with(wipe(180.0, 0.05), "wipe-progress", |output| {
        let early = rgb_at(output, 0.8, WIDTH / 2, HEIGHT / 2);
        let late = rgb_at(output, 1.2, WIDTH / 2, HEIGHT / 2);
        assert!(
            u16::from(early[0]) > u16::from(early[2]) + 100,
            "wipe starts from incoming instead of outgoing: {early:?}"
        );
        assert!(
            u16::from(late[2]) > u16::from(late[0]) + 100,
            "wipe returns to outgoing before completion: {late:?}"
        );
    });
}

fn patterned_frame(kind: TransitionKind) -> Vec<u8> {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    let first = pattern_clip("itm_pattern_a", 0, first_gradient());
    let transition = Transition {
        duration: time(500),
        alignment: TransitionAlignment::Centered,
        kind,
    };
    canonical.project.sequences[0].tracks.push(track(
        "trk_transition_pattern",
        TrackKind::Video,
        0,
        vec![
            first,
            pattern_clip("itm_pattern_b", 1_000, second_gradient()),
        ],
    ));
    add_transition(
        &mut canonical,
        "seq_main",
        "itm_pattern_a",
        "itm_pattern_b",
        transition,
    );
    let output = temp.path().join("transition-pattern.mp4");
    render(canonical, &BTreeMap::new(), &output);
    rgb_frame(&output, 1.0)
}

fn pattern_clip(id: &str, start_ms: i64, gradient: Gradient) -> Clip {
    let mut clip = solid_clip(id, color(0, 0, 0), start_ms, 1_000);
    clip.source = ClipSource::Generated {
        generator: Generator::Gradient { gradient },
    };
    clip
}

fn first_gradient() -> Gradient {
    Gradient::Linear {
        start: point(0.0, 0.0),
        end: point(1.0, 1.0),
        stops: vec![
            stop(0.0, color(255, 0, 0)),
            stop(0.45, color(0, 255, 255)),
            stop(1.0, color(0, 0, 255)),
        ],
    }
}

fn second_gradient() -> Gradient {
    Gradient::Radial {
        center: point(0.35, 0.65),
        radius: 0.7,
        stops: vec![
            stop(0.0, color(255, 255, 0)),
            stop(0.5, color(255, 0, 255)),
            stop(1.0, color(0, 153, 0)),
        ],
    }
}

fn point(x: f64, y: f64) -> Vec2 {
    Vec2 { x, y }
}

fn stop(offset: f64, color: Color) -> GradientStop {
    GradientStop { offset, color }
}

fn wipe(angle_degrees: f64, softness: f64) -> TransitionKind {
    TransitionKind::Wipe {
        direction: CardinalDirection::Left,
        angle_degrees,
        softness,
    }
}

fn circle(softness: f64) -> TransitionKind {
    TransitionKind::Circle {
        direction: CircleDirection::Open,
        softness,
    }
}

fn changed_channels(left: &[u8], right: &[u8], threshold: u8) -> usize {
    left.iter()
        .zip(right)
        .filter(|(a, b)| a.abs_diff(**b) > threshold)
        .count()
}

fn mean_channel(frame: &[u8]) -> f64 {
    frame.iter().map(|value| f64::from(*value)).sum::<f64>() / frame.len() as f64
}

fn horizontal_changes(frame: &[u8], threshold: u8) -> usize {
    let row_width = WIDTH as usize * 3;
    frame
        .chunks_exact(row_width)
        .map(|row| {
            row.chunks_exact(3)
                .zip(row.chunks_exact(3).skip(1))
                .filter(|(a, b)| a.iter().zip(*b).any(|(x, y)| x.abs_diff(*y) > threshold))
                .count()
        })
        .sum()
}
