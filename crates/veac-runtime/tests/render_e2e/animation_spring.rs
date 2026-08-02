use tempfile::tempdir;

use super::support::*;

const START_X: f64 = -24.0;
const END_X: f64 = 20.0;
const SPRING_END_SECONDS: f64 = 2.0;

#[test]
fn spring_position_executes_in_ffmpeg_and_matches_canonical_evaluation() {
    let temp = tempdir().unwrap();
    let output = temp.path().join("spring-position.mp4");
    let interpolation = spring();
    let rendered = render(
        spring_project(interpolation.clone()),
        &BTreeMap::new(),
        &output,
    );

    let graph = rendered.command.filter_graph.as_deref().unwrap();
    for term in ["exp(", "cos(", "sin("] {
        assert!(graph.contains(term), "missing {term} in graph:\n{graph}");
    }
    assert_media_contract(&output, 0, 3.0);

    let samples = [0.0, 0.5, 1.0, 1.5, 2.2];
    let centers: Vec<_> = samples
        .into_iter()
        .map(|second| {
            let actual = red_center_x(&rgb_frame(&output, second));
            let expected = expected_center_x(&interpolation, second);
            assert!(
                (actual - expected).abs() <= 1.5,
                "t={second}: actual={actual}, expected={expected}"
            );
            actual
        })
        .collect();

    let endpoint = f64::from(WIDTH) / 2.0 + END_X;
    assert!(centers[1] > centers[0] + 25.0, "centers={centers:?}");
    assert!(centers[2] > endpoint + 8.0, "no overshoot: {centers:?}");
    assert!((centers[4] - endpoint).abs() <= 1.0, "centers={centers:?}");
}

fn spring_project(interpolation: Interpolation) -> ProjectEnvelope {
    let mut value = project(false);
    let background = solid_clip("itm_spring_bg", color(0, 0, 0), 0, 3_000);
    let mut subject = solid_clip("itm_spring_subject", color(255, 0, 0), 0, 3_000);
    let mut visual = framed_visual(Anchor::Center, Some((12.0, 12.0)));
    visual.transform.position = Animatable::Keyframes {
        keyframes: vec![
            position_key("kf_spring_start", 0, START_X, interpolation),
            position_key("kf_spring_end", 2_000, END_X, Interpolation::Linear),
        ],
    };
    subject.visual = Some(visual);
    value.project.sequences[0].tracks.extend([
        track("trk_spring_bg", TrackKind::Video, 0, vec![background]),
        track("trk_spring_subject", TrackKind::Visual, 1, vec![subject]),
    ]);
    value
}

fn position_key(
    id: &str,
    milliseconds: i64,
    x: f64,
    interpolation: Interpolation,
) -> Keyframe<Point> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(milliseconds),
        value: Point {
            x: pixels(x),
            y: pixels(0.0),
        },
        interpolation,
    }
}

fn spring() -> Interpolation {
    Interpolation::Spring {
        frequency: 1.0,
        decay: 3.0,
        initial_velocity: 0.0,
    }
}

fn expected_center_x(interpolation: &Interpolation, second: f64) -> f64 {
    let progress = (second / SPRING_END_SECONDS).clamp(0.0, 1.0);
    let offset = START_X + (END_X - START_X) * interpolation.evaluate(progress);
    f64::from(WIDTH) / 2.0 + offset
}

fn red_center_x(frame: &[u8]) -> f64 {
    let mut minimum = WIDTH;
    let mut maximum = 0;
    let mut count = 0;
    for (index, pixel) in frame.chunks_exact(3).enumerate() {
        if pixel[0] > 140 && pixel[1] < 90 && pixel[2] < 90 {
            let x = index as u32 % WIDTH;
            minimum = minimum.min(x);
            maximum = maximum.max(x);
            count += 1;
        }
    }
    assert!(count >= 80, "red subject missing: count={count}");
    f64::from(minimum + maximum + 1) / 2.0
}
