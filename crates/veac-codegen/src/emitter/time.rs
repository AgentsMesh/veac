use veac_plan::canonical::{Rational, RationalTime, TimeRange};

pub(crate) fn seconds(time: RationalTime) -> String {
    decimal_ratio(i128::from(time.value), u128::from(time.timescale))
}

pub(crate) fn end(range: TimeRange) -> String {
    match range.end() {
        Ok(value) => seconds(value),
        Err(_) => "0".to_owned(),
    }
}

pub(crate) fn rate(value: Rational) -> String {
    decimal_ratio(i128::from(value.numerator), u128::from(value.denominator))
}

fn trim_decimal(mut value: String) -> String {
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    if value == "-0" {
        "0".to_owned()
    } else {
        value
    }
}

pub(crate) fn number(value: f64) -> String {
    trim_decimal(format!("{value:.12}"))
}

pub(crate) fn seconds_delta(left: RationalTime, right: RationalTime) -> String {
    if left.timescale == right.timescale {
        return decimal_ratio(
            i128::from(right.value) - i128::from(left.value),
            u128::from(left.timescale),
        );
    }
    let numerator = i128::from(right.value) * i128::from(left.timescale)
        - i128::from(left.value) * i128::from(right.timescale);
    let denominator = u128::from(left.timescale) * u128::from(right.timescale);
    decimal_ratio(numerator, denominator)
}

pub(crate) fn frame_window_start(end: RationalTime, frame_rate: Rational) -> String {
    let rate = i128::from(frame_rate.numerator);
    let scale = i128::from(end.timescale);
    let frames = 2 * i128::from(frame_rate.denominator) * scale;
    let numerator = (i128::from(end.value) * rate - frames).max(0);
    decimal_ratio(numerator, u128::try_from(scale * rate).unwrap_or(0))
}

fn decimal_ratio(numerator: i128, denominator: u128) -> String {
    const SCALE: u128 = 1_000_000_000_000;
    if numerator == 0 || denominator == 0 {
        return "0".to_owned();
    }
    let negative = numerator < 0;
    let magnitude = numerator.unsigned_abs();
    let mut whole = magnitude / denominator;
    let remainder = magnitude % denominator;
    let mut fraction = (remainder * SCALE + denominator / 2) / denominator;
    if fraction == SCALE {
        whole += 1;
        fraction = 0;
    }
    let sign = if negative { "-" } else { "" };
    if fraction == 0 {
        format!("{sign}{whole}")
    } else {
        trim_decimal(format!("{sign}{whole}.{fraction:012}"))
    }
}

pub(crate) fn samples(time: RationalTime, sample_rate: u32) -> Option<u64> {
    if time.value < 0 || time.timescale == 0 {
        return None;
    }
    let numerator = i128::from(time.value) * i128::from(sample_rate);
    let denominator = i128::from(time.timescale);
    if numerator % denominator != 0 {
        return None;
    }
    u64::try_from(numerator / denominator).ok()
}

pub(crate) fn containing_frame(time: RationalTime, frame_rate: Rational) -> Option<u64> {
    if time.value < 0 || time.timescale == 0 || !frame_rate.is_positive() {
        return None;
    }
    let numerator = u128::try_from(time.value).ok()? * u128::try_from(frame_rate.numerator).ok()?;
    let denominator = u128::from(time.timescale) * u128::from(frame_rate.denominator);
    u64::try_from(numerator / denominator).ok()
}
