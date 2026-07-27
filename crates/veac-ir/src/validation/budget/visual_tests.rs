use super::*;

#[test]
fn animated_scale_uses_each_axis_maximum() {
    let value = Animatable::Keyframes {
        keyframes: vec![
            Keyframe {
                id: KeyframeId::new("kf_budget_a").unwrap(),
                time: RationalTime::new(0, 600).unwrap(),
                value: Vec2 { x: 2.0, y: 3.0 },
                interpolation: Interpolation::Linear,
            },
            Keyframe {
                id: KeyframeId::new("kf_budget_b").unwrap(),
                time: RationalTime::new(1, 600).unwrap(),
                value: Vec2 { x: 4.0, y: 1.0 },
                interpolation: Interpolation::Linear,
            },
        ],
    };
    assert_eq!(max_scale(&value), Some((4.0, 3.0)));
    assert_eq!(
        max_scale(&Animatable::constant(Vec2 { x: 0.0, y: 1.0 })),
        None
    );
}
