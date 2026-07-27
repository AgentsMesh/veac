use super::*;
use crate::test_support::{range, time};

#[test]
fn absent_and_clip_bounded_enable_ranges_are_valid() {
    let mut absent = sample_project();
    effect_mut(&mut absent).enable_range = None;
    validate(&absent).unwrap();

    let mut exact = sample_project();
    effect_mut(&mut exact).enable_range = Some(range(0, 600));
    validate(&exact).unwrap();
}

#[test]
fn enable_ranges_reject_overrun_invalid_timebase_and_bad_duration() {
    let mut overrun = sample_project();
    effect_mut(&mut overrun).enable_range = Some(range(599, 2));
    assert_effect_code(&overrun, "EFFECT_RANGE");

    let mut wrong_base = sample_project();
    effect_mut(&mut wrong_base).enable_range = Some(TimeRange {
        start: RationalTime {
            value: 0,
            timescale: 1,
        },
        duration: RationalTime {
            value: 1,
            timescale: 1,
        },
    });
    assert_effect_code(&wrong_base, "EFFECT_RANGE");

    let mut zero = sample_project();
    effect_mut(&mut zero).enable_range = Some(TimeRange {
        start: time(0),
        duration: time(0),
    });
    assert_effect_code(&zero, "EFFECT_RANGE");

    let mut overflow = sample_project();
    effect_mut(&mut overflow).enable_range = Some(TimeRange {
        start: time(MAX_SAFE_INTEGER as i64),
        duration: time(1),
    });
    assert_effect_code(&overflow, "EFFECT_RANGE");
}
