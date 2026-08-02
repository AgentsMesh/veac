use super::*;

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

fn number_keys(interpolation: Interpolation) -> Vec<Keyframe<f64>> {
    vec![
        Keyframe {
            id: KeyframeId::new("kf_left").unwrap(),
            time: time(0),
            value: 0.0,
            interpolation,
        },
        Keyframe {
            id: KeyframeId::new("kf_right").unwrap(),
            time: time(100),
            value: 1.0,
            interpolation: Interpolation::Hold,
        },
    ]
}

fn sampled(interpolation: Interpolation) -> f64 {
    at(&number_keys(interpolation), time(25)).unwrap()
}

#[test]
fn scalar_curves_cover_boundaries_extrapolation_and_every_easing() {
    assert_eq!(at::<f64>(&[], time(0)), None);
    let keys = number_keys(Interpolation::Linear);
    assert_eq!(at(&keys, time(-1)), Some(0.0));
    assert_eq!(at(&keys, time(100)), Some(1.0));
    assert_eq!(at(&keys, time(101)), Some(1.0));

    assert_eq!(sampled(Interpolation::Hold), 0.0);
    assert_eq!(sampled(Interpolation::Linear), 0.25);
    assert_eq!(sampled(Interpolation::EaseIn), 0.0625);
    assert_eq!(sampled(Interpolation::EaseOut), 0.4375);
    assert_eq!(sampled(Interpolation::EaseInOut), 0.15625);
    let bezier = sampled(Interpolation::CubicBezier {
        x1: 0.42,
        y1: 0.0,
        x2: 0.58,
        y2: 1.0,
    });
    assert!((bezier - 0.129_161_9).abs() < 0.000_01);
}

#[test]
fn spring_is_endpoint_normalized_and_overshoots() {
    let spring = Interpolation::Spring {
        frequency: 1.5,
        decay: 6.0,
        initial_velocity: 0.0,
    };
    assert_close(spring.evaluate(0.0), 0.0);
    assert_close(spring.evaluate(1.0), 1.0);
    assert_close(spring.spring_derivative(0.0).unwrap(), 0.0);
    assert!((1..100)
        .map(|step| spring.evaluate(f64::from(step) / 100.0))
        .any(|value| value > 1.0));
}

#[test]
fn spring_stability_bounds_are_closed_and_invalid_values_fail_evaluation() {
    for spring in [spring(64.0, 0.0), spring(6.0, 128.0), spring(6.0, -128.0)] {
        assert!(spring.spring_coefficients().is_some());
        assert_close(spring.evaluate(0.0), 0.0);
        assert_close(spring.evaluate(1.0), 1.0);
        assert!(spring.evaluate(0.5).is_finite());
    }
    for spring in [
        spring(64.000_1, 0.0),
        spring(6.0, 128.000_1),
        spring(6.0, -128.000_1),
    ] {
        assert!(spring.spring_coefficients().is_none());
        assert!(spring.evaluate(0.5).is_nan());
    }
}

#[test]
fn vectors_and_compatible_points_interpolate_component_wise() {
    let vectors = vec![
        key("kf_v0", Vec2 { x: 0.0, y: 4.0 }, 0),
        key("kf_v1", Vec2 { x: 8.0, y: 0.0 }, 100),
    ];
    assert_eq!(at(&vectors, time(25)), Some(Vec2 { x: 2.0, y: 3.0 }));

    let points = vec![
        key("kf_p0", point(0.0, 4.0, LengthUnit::Pixels), 0),
        key("kf_p1", point(8.0, 0.0, LengthUnit::Pixels), 100),
    ];
    assert_eq!(
        at(&points, time(25)),
        Some(point(2.0, 3.0, LengthUnit::Pixels))
    );
}

#[test]
fn points_with_incompatible_units_fail_closed() {
    let left = point(0.0, 0.0, LengthUnit::Pixels);
    let x_mismatch = Point {
        x: Length {
            value: 1.0,
            unit: LengthUnit::Normalized,
        },
        y: left.y,
    };
    let y_mismatch = Point {
        x: left.x,
        y: Length {
            value: 1.0,
            unit: LengthUnit::Percent,
        },
    };
    for right in [x_mismatch, y_mismatch] {
        let keys = vec![key("kf_p0", left, 0), key("kf_p1", right, 100)];
        assert_eq!(at(&keys, time(50)), None);
    }
}

fn key<T>(id: &str, value: T, at: i64) -> Keyframe<T> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: time(at),
        value,
        interpolation: Interpolation::Linear,
    }
}

fn spring(decay: f64, initial_velocity: f64) -> Interpolation {
    Interpolation::Spring {
        frequency: 1.5,
        decay,
        initial_velocity,
    }
}

fn point(x: f64, y: f64, unit: LengthUnit) -> Point {
    Point {
        x: Length { value: x, unit },
        y: Length { value: y, unit },
    }
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}
