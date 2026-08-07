use crate::program::expression::{ExactNumber, Value};

use super::super::time;

#[test]
fn time_conversion_closures_execute_in_the_test_build() {
    let denominator_overflow =
        Value::Time(ExactNumber::new(1, i128::from(u32::MAX) + 1).expect("positive denominator"));
    assert!(time::coordinate(Some(&denominator_overflow), 600).is_err());

    let product_overflow = Value::Time(ExactNumber::integer(i128::MAX));
    assert!(time::coordinate(Some(&product_overflow), u32::MAX).is_err());

    let tick_overflow = Value::Time(ExactNumber::integer(i128::from(i64::MAX) + 1));
    assert!(time::coordinate(Some(&tick_overflow), 1).is_err());

    let zero_timescale = Value::Time(ExactNumber::integer(1));
    assert!(time::coordinate(Some(&zero_timescale), 0).is_err());

    assert!(time::intrinsic(Some(&tick_overflow)).is_err());
    assert!(time::intrinsic(Some(&denominator_overflow)).is_err());
    assert_eq!(time::intrinsic(Some(&zero_timescale)).unwrap().value, 1);
}
