use veac_ir::{Rational, RationalTime, TimeRange, MAX_SAFE_INTEGER};

pub(super) fn source_range(
    source_start: RationalTime,
    record_duration: RationalTime,
    rate: Rational,
    repeat: u32,
) -> Option<TimeRange> {
    let numerator = i128::from(record_duration.value).checked_mul(i128::from(rate.numerator))?;
    let denominator = u128::from(record_duration.timescale)
        .checked_mul(u128::from(rate.denominator))?
        .checked_mul(u128::from(repeat))?;
    let divisor = gcd(numerator.unsigned_abs(), denominator);
    let value = numerator / i128::try_from(divisor).ok()?;
    let timescale = denominator / divisor;
    let duration = RationalTime {
        value: i64::try_from(value).ok()?,
        timescale: u32::try_from(timescale).ok()?,
    };
    aligned_range(source_start, duration)
}

pub(super) fn range_between(start: RationalTime, end: RationalTime) -> Option<TimeRange> {
    let common = lcm(start.timescale, end.timescale)?;
    let start_value = i128::from(start.value) * i128::from(common / start.timescale);
    let end_value = i128::from(end.value) * i128::from(common / end.timescale);
    Some(TimeRange {
        start: RationalTime {
            value: i64::try_from(start_value).ok()?,
            timescale: common,
        },
        duration: RationalTime {
            value: i64::try_from(end_value.checked_sub(start_value)?).ok()?,
            timescale: common,
        },
    })
}

fn aligned_range(start: RationalTime, duration: RationalTime) -> Option<TimeRange> {
    let common = lcm(start.timescale, duration.timescale)?;
    let start_value = i128::from(start.value) * i128::from(common / start.timescale);
    let duration_value = i128::from(duration.value) * i128::from(common / duration.timescale);
    if start_value.unsigned_abs() > u128::from(MAX_SAFE_INTEGER)
        || duration_value <= 0
        || duration_value.unsigned_abs() > u128::from(MAX_SAFE_INTEGER)
    {
        return None;
    }
    Some(TimeRange {
        start: RationalTime::new(i64::try_from(start_value).ok()?, common).ok()?,
        duration: RationalTime::new(i64::try_from(duration_value).ok()?, common).ok()?,
    })
}

fn lcm(left: u32, right: u32) -> Option<u32> {
    if left == 0 || right == 0 {
        return None;
    }
    let divisor = gcd(u128::from(left), u128::from(right));
    let value = u128::from(left) / divisor * u128::from(right);
    u32::try_from(value).ok()
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}
