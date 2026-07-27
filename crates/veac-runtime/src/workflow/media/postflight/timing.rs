use veac_ir::{Rational, RationalTime};

const TOLERANCE_MICROSECONDS: u128 = 1_000;
const MICROS_PER_SECOND: u128 = 1_000_000;

pub(crate) fn video(actual: RationalTime, expected: RationalTime, rate: Rational) -> bool {
    u128::try_from(rate.numerator)
        .ok()
        .is_some_and(|denominator| {
            within(actual, expected, u128::from(rate.denominator), denominator)
        })
}

pub(crate) fn audio(actual: RationalTime, expected: RationalTime, sample_rate: u32) -> bool {
    within(actual, expected, 1, u128::from(sample_rate))
}

fn within(
    actual: RationalTime,
    expected: RationalTime,
    slack_numerator: u128,
    slack_denominator: u128,
) -> bool {
    let Some(actual_value) = nonnegative(actual) else {
        return false;
    };
    let Some(expected_value) = nonnegative(expected) else {
        return false;
    };
    let actual_scale = u128::from(actual.timescale);
    let expected_scale = u128::from(expected.timescale);
    lower_bound(actual_value, actual_scale, expected_value, expected_scale)
        && upper_bound(
            actual_value,
            actual_scale,
            expected_value,
            expected_scale,
            slack_numerator,
            slack_denominator,
        )
}

fn lower_bound(actual: u128, actual_scale: u128, expected: u128, expected_scale: u128) -> bool {
    let left = actual
        .checked_mul(expected_scale)
        .and_then(|value| value.checked_mul(MICROS_PER_SECOND))
        .and_then(|value| {
            TOLERANCE_MICROSECONDS
                .checked_mul(actual_scale)
                .and_then(|tolerance| tolerance.checked_mul(expected_scale))
                .and_then(|tolerance| value.checked_add(tolerance))
        });
    let right = expected
        .checked_mul(actual_scale)
        .and_then(|value| value.checked_mul(MICROS_PER_SECOND));
    left.zip(right).is_some_and(|(left, right)| left >= right)
}

fn upper_bound(
    actual: u128,
    actual_scale: u128,
    expected: u128,
    expected_scale: u128,
    slack_numerator: u128,
    slack_denominator: u128,
) -> bool {
    let left = actual
        .checked_mul(expected_scale)
        .and_then(|value| value.checked_mul(slack_denominator))
        .and_then(|value| value.checked_mul(MICROS_PER_SECOND));
    let expected = expected
        .checked_mul(actual_scale)
        .and_then(|value| value.checked_mul(slack_denominator))
        .and_then(|value| value.checked_mul(MICROS_PER_SECOND));
    let quantum = slack_numerator
        .checked_mul(actual_scale)
        .and_then(|value| value.checked_mul(expected_scale))
        .and_then(|value| value.checked_mul(MICROS_PER_SECOND));
    let tolerance = TOLERANCE_MICROSECONDS
        .checked_mul(actual_scale)
        .and_then(|value| value.checked_mul(expected_scale))
        .and_then(|value| value.checked_mul(slack_denominator));
    expected
        .zip(quantum)
        .and_then(|(expected, quantum)| expected.checked_add(quantum))
        .zip(tolerance)
        .and_then(|(value, tolerance)| value.checked_add(tolerance))
        .zip(left)
        .is_some_and(|(right, left)| left <= right)
}

fn nonnegative(value: RationalTime) -> Option<u128> {
    value
        .is_valid()
        .then(|| u128::try_from(value.value).ok())
        .flatten()
}

#[cfg(test)]
#[path = "timing/tests.rs"]
mod tests;
