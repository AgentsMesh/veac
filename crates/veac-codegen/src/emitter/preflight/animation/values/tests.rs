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
fn spring_range_validation_checks_intermediate_overshoot() {
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
    assert!(!spring_ranges_valid(&keyframes, |value| {
        value.x <= 16.0 && value.y <= 16.0
    }));
}
