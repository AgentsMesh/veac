use super::*;

#[test]
fn restricted_named_and_asymmetric_bezier_easings_are_equivalent() {
    let easings = [
        Interpolation::EaseIn,
        Interpolation::EaseOut,
        Interpolation::EaseInOut,
        Interpolation::CubicBezier {
            x1: 0.08,
            y1: 0.2,
            x2: 0.82,
            y2: 0.9,
        },
    ];
    for easing in easings {
        let keys = keys(easing.clone());
        let sliced = between(&keys, time(17), time(83)).unwrap();
        let low = easing.evaluate(0.17);
        let high = easing.evaluate(0.83);
        for local in [0.0, 0.1, 0.37, 0.75, 1.0] {
            let expected = (easing.evaluate(0.17 + 0.66 * local) - low) / (high - low);
            assert_close(sliced.evaluate(local), expected);
        }
        let Interpolation::CubicBezier { x1, x2, .. } = sliced else {
            panic!("partial nonlinear easing must become cubic Bezier");
        };
        assert!((0.0..=1.0).contains(&x1) && (0.0..=1.0).contains(&x2));
    }
}

#[test]
fn full_linear_and_flat_ranges_keep_their_exact_forms() {
    assert_eq!(
        between(&keys(Interpolation::EaseIn), time(0), time(100)).unwrap(),
        Interpolation::EaseIn
    );
    assert_eq!(
        between(&keys(Interpolation::Linear), time(20), time(80)).unwrap(),
        Interpolation::Linear
    );
    assert_eq!(
        between(&keys(Interpolation::EaseOut), time(-20), time(0)).unwrap(),
        Interpolation::Hold
    );
}

#[test]
fn nonflat_subcurve_with_equal_endpoints_is_not_silently_flattened() {
    let easing = Interpolation::CubicBezier {
        x1: 1.0 / 3.0,
        y1: 1.0,
        x2: 2.0 / 3.0,
        y2: -4.0 / 3.0,
    };
    assert!(between(&keys(easing), time(0), time(50)).is_err());
}

fn keys(interpolation: Interpolation) -> Vec<Keyframe<f64>> {
    vec![
        Keyframe {
            id: KeyframeId::new("kf_start").unwrap(),
            time: time(0),
            value: 0.0,
            interpolation,
        },
        Keyframe {
            id: KeyframeId::new("kf_end").unwrap(),
            time: time(100),
            value: 1.0,
            interpolation: Interpolation::Linear,
        },
    ]
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-10, "{actual} != {expected}");
}
