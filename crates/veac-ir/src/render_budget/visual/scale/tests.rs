use crate::{Animatable, Interpolation, Keyframe, KeyframeId, RationalTime, Vec2};

use super::max_visual_scale;

#[test]
fn cubic_overshoot_contributes_to_the_maximum_scale() {
    let curve = Animatable::Keyframes {
        keyframes: vec![key("kf_scale_start", 0, 1.0), key("kf_scale_end", 600, 2.0)],
    };
    let maximum = max_visual_scale(&curve).unwrap();
    assert!((maximum.0 - 2.058_406_766).abs() < 1e-6);
    assert!((maximum.1 - 2.058_406_766).abs() < 1e-6);
}

fn key(id: &str, at: i64, value: f64) -> Keyframe<Vec2> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: RationalTime::new(at, 600).unwrap(),
        value: Vec2 { x: value, y: value },
        interpolation: Interpolation::CubicBezier {
            x1: 0.2,
            y1: -0.4,
            x2: 0.8,
            y2: 1.4,
        },
    }
}
