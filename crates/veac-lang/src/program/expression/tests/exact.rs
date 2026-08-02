use std::cmp::Ordering;

use crate::program::expression::ExactNumber;

#[test]
fn exact_numbers_normalize_and_render_without_floating_point() {
    let half = ExactNumber::new(2, 4).unwrap();
    assert_eq!(half.numerator(), 1);
    assert_eq!(half.denominator(), 2);
    assert_eq!(half.render(), "0.5");
    assert_eq!(ExactNumber::new(1, 3).unwrap().render(), "1 / 3");
    assert_eq!(ExactNumber::new(1, 40).unwrap().render(), "0.025");
    assert_eq!(ExactNumber::new(-2, -4).unwrap().render(), "0.5");
    assert_eq!(ExactNumber::new(0, 9), Some(ExactNumber::integer(0)));
    assert!(ExactNumber::new(1, 0).is_none());
    assert!(ExactNumber::new(i128::MIN, -1).is_none());
}

#[test]
fn exact_arithmetic_reduces_before_operating() {
    let third = ExactNumber::new(1, 3).unwrap();
    let sixth = ExactNumber::new(1, 6).unwrap();
    assert_eq!(third.checked_add(sixth).unwrap().render(), "0.5");
    assert_eq!(third.checked_sub(sixth).unwrap().render(), "1 / 6");
    assert_eq!(
        third.checked_mul(ExactNumber::integer(3)).unwrap().render(),
        "1"
    );
    assert_eq!(third.checked_div(sixth).unwrap().render(), "2");
    assert_eq!(
        third
            .checked_div(ExactNumber::integer(-2))
            .unwrap()
            .render(),
        "-1 / 6"
    );
    assert_eq!(
        ExactNumber::integer(i128::MIN).checked_div(ExactNumber::integer(i128::MIN)),
        Some(ExactNumber::integer(1))
    );
    assert!(ExactNumber::integer(1)
        .checked_div(ExactNumber::integer(i128::MIN))
        .is_none());
    assert!(third.checked_div(ExactNumber::integer(0)).is_none());
    assert_eq!(third.checked_neg().unwrap().render(), "-1 / 3");
}

#[test]
fn addition_and_subtraction_narrow_only_after_wide_normalization() {
    let half_max = ExactNumber::new(i128::MAX, 2).unwrap();
    assert_eq!(
        half_max.checked_add(half_max),
        Some(ExactNumber::integer(i128::MAX))
    );
    let minimum = ExactNumber::integer(i128::MIN);
    assert_eq!(minimum.checked_sub(minimum), Some(ExactNumber::integer(0)));
    assert!(ExactNumber::integer(i128::MAX)
        .checked_add(ExactNumber::integer(1))
        .is_none());
    assert!(ExactNumber::integer(i128::MIN)
        .checked_sub(ExactNumber::integer(1))
        .is_none());
}

#[test]
fn comparison_is_exact_even_when_cross_products_would_overflow() {
    let large = ExactNumber::new(i128::MAX, i128::MAX - 1).unwrap();
    let one = ExactNumber::integer(1);
    assert_eq!(large.compare(one), Ordering::Greater);
    assert_eq!(one.compare(large), Ordering::Less);
    assert_eq!(one.compare(one), Ordering::Equal);
    assert_eq!(
        ExactNumber::integer(0).compare(ExactNumber::integer(0)),
        Ordering::Equal
    );
    assert_eq!(
        ExactNumber::integer(-2).compare(ExactNumber::integer(-1)),
        Ordering::Less
    );
    assert_eq!(
        ExactNumber::integer(-1).compare(ExactNumber::integer(0)),
        Ordering::Less
    );
    let half = ExactNumber::new(1, 2).unwrap();
    let two_fifths = ExactNumber::new(2, 5).unwrap();
    assert_eq!(half.compare(two_fifths), Ordering::Greater);
    assert_eq!(two_fifths.compare(half), Ordering::Less);
}
