use crate::program::expression::ExactNumber;

pub(super) fn decimal(raw: &str) -> Option<ExactNumber> {
    if raw.is_empty() || raw.trim() != raw {
        return None;
    }
    let (negative, raw) = raw
        .strip_prefix('-')
        .map_or((false, raw), |value| (true, value));
    let (whole, fraction) = raw.split_once('.').unwrap_or((raw, ""));
    if whole.is_empty()
        || !whole.bytes().all(|value| value.is_ascii_digit())
        || !fraction.bytes().all(|value| value.is_ascii_digit())
        || (raw.contains('.') && fraction.is_empty())
    {
        return None;
    }
    let denominator = 10_i128.checked_pow(u32::try_from(fraction.len()).ok()?)?;
    let whole = whole.parse::<i128>().ok()?;
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i128>().ok()?
    };
    let numerator = whole.checked_mul(denominator)?.checked_add(fraction)?;
    ExactNumber::new(
        if negative {
            numerator.checked_neg()?
        } else {
            numerator
        },
        denominator,
    )
}

pub(super) fn unit(raw: &str, suffix: &str, scale: (u64, u64)) -> Option<ExactNumber> {
    let number = raw.strip_suffix(suffix)?;
    let value = decimal(number)?;
    let numerator = value.numerator().checked_mul(i128::from(scale.0))?;
    let denominator = value.denominator().checked_mul(i128::from(scale.1))?;
    ExactNumber::new(numerator, denominator)
}
