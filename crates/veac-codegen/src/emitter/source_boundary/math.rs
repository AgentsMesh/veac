use veac_plan::canonical::RationalTime;

pub(super) fn difference(greater: RationalTime, lesser: RationalTime) -> Option<RationalTime> {
    let scale = lcm(greater.timescale, lesser.timescale)?;
    let greater = scaled(greater, scale)?;
    let lesser = scaled(lesser, scale)?;
    RationalTime::new(i64::try_from(greater.checked_sub(lesser)?).ok()?, scale).ok()
}

pub(super) fn tick_after(value: RationalTime) -> Option<RationalTime> {
    RationalTime::new(1, value.timescale).ok()
}

fn scaled(value: RationalTime, scale: u32) -> Option<i128> {
    i128::from(value.value).checked_mul(i128::from(scale.checked_div(value.timescale)?))
}

fn lcm(left: u32, right: u32) -> Option<u32> {
    let divisor = gcd(left, right);
    left.checked_div(divisor)?.checked_mul(right)
}

fn gcd(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}
