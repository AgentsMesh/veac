use super::*;
use veac_plan::canonical::{KeyframeId, RationalTime};

#[test]
fn spring_interpolation_requires_valid_coefficients() {
    assert!(interpolation(&Interpolation::Spring {
        frequency: 1.5,
        decay: 6.0,
        initial_velocity: 0.0,
    }));
    assert!(!interpolation(&Interpolation::Spring {
        frequency: 0.0,
        decay: 6.0,
        initial_velocity: 0.0,
    }));
}

#[test]
fn cubic_interpolation_allows_finite_y_overshoot_but_bounds_x() {
    assert!(interpolation(&Interpolation::CubicBezier {
        x1: 0.2,
        y1: -0.4,
        x2: 0.8,
        y2: 1.4,
    }));
    assert!(!interpolation(&Interpolation::CubicBezier {
        x1: -0.1,
        y1: 0.0,
        x2: 0.8,
        y2: 1.0,
    }));
}

#[test]
fn interpolation_range_validation_checks_spring_overshoot() {
    let keyframes = vec![
        Keyframe {
            id: KeyframeId::new("kf_spring_range_a").unwrap(),
            time: RationalTime::new(0, 600).unwrap(),
            value: Vec2 { x: 1.0, y: 1.0 },
            interpolation: Interpolation::Spring {
                frequency: 1.5,
                decay: 6.0,
                initial_velocity: 0.0,
            },
        },
        Keyframe {
            id: KeyframeId::new("kf_spring_range_b").unwrap(),
            time: RationalTime::new(600, 600).unwrap(),
            value: Vec2 { x: 16.0, y: 16.0 },
            interpolation: Interpolation::Linear,
        },
    ];
    assert!(!interpolation_ranges_valid(&keyframes, |value| {
        value.x <= 16.0 && value.y <= 16.0
    }));
}

#[test]
fn interpolation_range_validation_accepts_safe_cubic_overshoot() {
    let mut keyframes = scalar_curve(0.6, 0.8);
    assert!(interpolation_ranges_valid(&keyframes, unit_number));
    keyframes[0].value = 0.0;
    keyframes[1].value = 1.0;
    assert!(!interpolation_ranges_valid(&keyframes, unit_number));
}

fn scalar_curve(start: f64, end: f64) -> Vec<Keyframe<f64>> {
    vec![
        Keyframe {
            id: KeyframeId::new("kf_cubic_range_a").unwrap(),
            time: RationalTime::new(0, 600).unwrap(),
            value: start,
            interpolation: Interpolation::CubicBezier {
                x1: 0.2,
                y1: -0.4,
                x2: 0.8,
                y2: 1.4,
            },
        },
        Keyframe {
            id: KeyframeId::new("kf_cubic_range_b").unwrap(),
            time: RationalTime::new(600, 600).unwrap(),
            value: end,
            interpolation: Interpolation::Linear,
        },
    ]
}
