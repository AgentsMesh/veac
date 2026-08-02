use veac_ir::{RationalTime, TimeRange};

pub(super) fn range(start: RationalTime, duration: RationalTime) -> Option<TimeRange> {
    let end = sum(start, duration)?;
    super::super::super::time::range_between(start, end)
}

pub(super) fn curve_offset(
    value: RationalTime,
    delta: RationalTime,
    duration: RationalTime,
) -> Option<RationalTime> {
    rational(
        i128::from(value.value) * i128::from(delta.value) * i128::from(duration.timescale),
        u128::from(value.timescale)
            * u128::from(delta.timescale)
            * u128::try_from(duration.value).ok()?,
    )
}

pub(super) fn scale(
    value: RationalTime,
    numerator: i128,
    denominator: u128,
) -> Option<RationalTime> {
    rational(
        i128::from(value.value) * numerator,
        u128::from(value.timescale) * denominator,
    )
}

pub(super) fn sum(left: RationalTime, right: RationalTime) -> Option<RationalTime> {
    sum_signed(left, right)
}

pub(super) fn sum_signed(left: RationalTime, right: RationalTime) -> Option<RationalTime> {
    rational(
        i128::from(left.value) * i128::from(right.timescale)
            + i128::from(right.value) * i128::from(left.timescale),
        u128::from(left.timescale) * u128::from(right.timescale),
    )
}

pub(super) fn difference(left: RationalTime, right: RationalTime) -> Option<RationalTime> {
    let value = difference_signed(left, right)?;
    (value.value >= 0).then_some(value)
}

pub(super) fn difference_signed(left: RationalTime, right: RationalTime) -> Option<RationalTime> {
    sum_signed(
        left,
        RationalTime {
            value: -right.value,
            timescale: right.timescale,
        },
    )
}

fn rational(numerator: i128, denominator: u128) -> Option<RationalTime> {
    let divisor = gcd(numerator.unsigned_abs(), denominator);
    RationalTime::new(
        i64::try_from(numerator / i128::try_from(divisor).ok()?).ok()?,
        u32::try_from(denominator / divisor).ok()?,
    )
    .ok()
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}
