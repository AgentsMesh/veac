use crate::test_support::time;

use super::*;

#[test]
fn cubic_bezier_y_overshoot_is_canonical() {
    let project = with_interpolation(Interpolation::CubicBezier {
        x1: 0.25,
        y1: -0.4,
        x2: 0.75,
        y2: 1.4,
    });
    validate(&project).unwrap();
}

#[test]
fn cubic_bezier_output_must_stay_inside_the_value_domain() {
    let project = with_interpolation(Interpolation::CubicBezier {
        x1: 0.25,
        y1: -2.0,
        x2: 0.75,
        y2: 3.0,
    });
    assert_code(&validation_codes(&project), "ANIMATION_VALUE");
}

#[test]
fn cubic_bezier_requires_finite_components_and_bounded_x() {
    for interpolation in [
        Interpolation::CubicBezier {
            x1: -0.1,
            y1: 0.0,
            x2: 0.5,
            y2: 1.0,
        },
        Interpolation::CubicBezier {
            x1: 0.5,
            y1: f64::INFINITY,
            x2: 0.5,
            y2: 1.0,
        },
    ] {
        let codes = validation_codes(&with_interpolation(interpolation));
        assert_code(&codes, "BEZIER");
    }
}

fn with_interpolation(interpolation: Interpolation) -> ProjectEnvelope {
    let mut project = sample_project();
    let ClipSource::Caption { style, .. } =
        &mut project.project.sequences[0].tracks[1].clips[0].source
    else {
        panic!("caption fixture")
    };
    style.animation = Some(TextAnimation {
        granularity: TextGranularity::Whole,
        transform: TextUnitTransform::default(),
        reveal: Animatable::constant(1.0),
        highlight: None,
        opacity: Animatable::Keyframes {
            keyframes: vec![
                key("kf_bezier_start", 0, 0.2, interpolation),
                key("kf_bezier_end", 100, 0.8, Interpolation::Linear),
            ],
        },
        stagger: time(0),
    });
    project
}

fn key(id: &str, at: i64, value: f64, interpolation: Interpolation) -> Keyframe<f64> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation,
    }
}
