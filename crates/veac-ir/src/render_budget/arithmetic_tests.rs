use super::*;

#[test]
fn duration_work_uses_exact_ceil_and_fails_closed() {
    let time = RationalTime::new(1, 3).unwrap();
    assert_eq!(
        units_for_duration(time, Rational::new(30, 1).unwrap()),
        Some(10)
    );
    assert_eq!(samples_for_duration(time, 8), Some(3));
    assert_eq!(channel_samples_for_duration(time, 8, 2), Some(6));
    assert!(!duration_within_seconds(
        RationalTime {
            value: 1,
            timescale: 0
        },
        1
    ));
    assert_eq!(units_for_duration(time, Rational::new(0, 1).unwrap()), None);
    assert_eq!(samples_for_duration(time, 0), None);
    assert_eq!(channel_samples_for_duration(time, 8, 0), None);
    assert_eq!(ceil_div(1, 0), u128::MAX);
}

#[test]
fn products_saturate_and_scaled_duration_is_exact() {
    assert_eq!(pixel_frames(u128::MAX, 2, 2), u128::MAX);
    let second = RationalTime::new(600, 600).unwrap();
    assert!(scaled_duration_within_seconds(
        second,
        Rational::new(30, 1).unwrap(),
        1,
        30
    ));
    assert!(!scaled_duration_within_seconds(
        second,
        Rational::new(31, 1).unwrap(),
        1,
        30
    ));
    assert!(!scaled_duration_within_seconds(
        second,
        Rational::new(1, 1).unwrap(),
        0,
        30
    ));
}
