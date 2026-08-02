use super::*;
use veac_plan::canonical::{Interpolation, KeyframeId, Length, LengthUnit, RationalTime};

#[test]
fn empty_and_between_keyframes_use_typed_defaults_and_pair_interpolation() {
    assert_eq!(number(&keyframes(vec![]), 0.5), 0.0);
    assert_eq!(
        point(
            &Animatable::Keyframes { keyframes: vec![] },
            0.5,
            (100, 100)
        ),
        (0.0, 0.0)
    );
    assert_eq!(
        vec2(&Animatable::Keyframes { keyframes: vec![] }, 0.5),
        (1.0, 1.0)
    );

    let points = Animatable::Keyframes {
        keyframes: vec![
            frame("kf_a", point_value(0.0, 10.0), 0),
            frame("kf_b", point_value(20.0, 30.0), 600),
        ],
    };
    assert_eq!(point(&points, 0.5, (100, 100)), (10.0, 20.0));
    let vectors = Animatable::Keyframes {
        keyframes: vec![
            frame("kf_c", Vec2 { x: 1.0, y: 2.0 }, 0),
            frame("kf_d", Vec2 { x: 3.0, y: 4.0 }, 600),
        ],
    };
    assert_eq!(vec2(&vectors, 0.5), (2.0, 3.0));
}

#[test]
fn samples_after_the_last_spatial_keyframe_hold_the_final_value() {
    let points = Animatable::Keyframes {
        keyframes: vec![
            frame("kf_point_a", point_value(0.0, 10.0), 0),
            frame("kf_point_b", point_value(20.0, 30.0), 600),
        ],
    };
    assert_eq!(point(&points, 2.0, (100, 100)), (20.0, 30.0));

    let vectors = Animatable::Keyframes {
        keyframes: vec![
            frame("kf_vector_a", Vec2 { x: 1.0, y: 2.0 }, 0),
            frame("kf_vector_b", Vec2 { x: 3.0, y: 4.0 }, 600),
        ],
    };
    assert_eq!(vec2(&vectors, 2.0), (3.0, 4.0));
}

fn keyframes(values: Vec<Keyframe<f64>>) -> Animatable<f64> {
    Animatable::Keyframes { keyframes: values }
}

fn frame<T>(id: &str, value: T, ticks: i64) -> Keyframe<T> {
    Keyframe {
        id: KeyframeId::new(id).unwrap(),
        time: RationalTime::new(ticks, 600).unwrap(),
        value,
        interpolation: Interpolation::Linear,
    }
}

fn point_value(x: f64, y: f64) -> Point {
    Point {
        x: Length {
            value: x,
            unit: LengthUnit::Pixels,
        },
        y: Length {
            value: y,
            unit: LengthUnit::Pixels,
        },
    }
}
