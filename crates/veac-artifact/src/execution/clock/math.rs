use veac_ir::{RationalTime, TimeRange};

pub(super) fn mapped_time(
    logical: RationalTime,
    origin: RationalTime,
    physical: RationalTime,
) -> Option<RationalTime> {
    let scale = common_scale(&[logical, origin, physical])?;
    let logical = scaled(logical, scale)?;
    let origin = scaled(origin, scale)?;
    let physical = scaled(physical, scale)?;
    let mapped = physical.checked_add(logical.checked_sub(origin)?)?;
    RationalTime::new(i64::try_from(mapped).ok()?, scale).ok()
}

pub(super) fn stream_domain(
    duration: RationalTime,
    physical_start: RationalTime,
) -> Option<(TimeRange, RationalTime)> {
    if duration.value <= 0 || physical_start.value < 0 {
        return None;
    }
    let scale = common_scale(&[duration, physical_start])?;
    let duration = time_at_scale(duration, scale)?;
    let physical_start = time_at_scale(physical_start, scale)?;
    let range = TimeRange::new(RationalTime::zero(scale).ok()?, duration).ok()?;
    Some((range, physical_start))
}

pub(super) fn extended_range(
    range: TimeRange,
    before: RationalTime,
    after: RationalTime,
) -> Option<TimeRange> {
    if before.value < 0 || after.value < 0 {
        return None;
    }
    let values = [range.start, range.duration, before, after];
    let scale = common_scale(&values)?;
    let start = scaled(range.start, scale)?.checked_sub(scaled(before, scale)?)?;
    let duration = scaled(range.duration, scale)?
        .checked_add(scaled(before, scale)?)?
        .checked_add(scaled(after, scale)?)?;
    TimeRange::new(
        RationalTime::new(i64::try_from(start).ok()?, scale).ok()?,
        RationalTime::new(i64::try_from(duration).ok()?, scale).ok()?,
    )
    .ok()
}

fn time_at_scale(value: RationalTime, scale: u32) -> Option<RationalTime> {
    RationalTime::new(i64::try_from(scaled(value, scale)?).ok()?, scale).ok()
}

fn common_scale(values: &[RationalTime]) -> Option<u32> {
    values
        .iter()
        .try_fold(1, |scale, value| lcm(scale, value.timescale))
}

fn scaled(value: RationalTime, scale: u32) -> Option<i128> {
    i128::from(value.value).checked_mul(i128::from(scale.checked_div(value.timescale)?))
}

fn lcm(left: u32, right: u32) -> Option<u32> {
    if left == 0 || right == 0 {
        return None;
    }
    let divisor = gcd(u64::from(left), u64::from(right));
    u32::try_from(u64::from(left) / divisor * u64::from(right)).ok()
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}
