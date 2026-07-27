use crate::authoring::{NumberLiteral, Span};
use veac_ir::{Rational, RationalTime, TimeRange};

use super::context::Context;

pub fn time(ctx: &mut Context, value: &NumberLiteral) -> Option<RationalTime> {
    let (number, unit) = split(&value.raw);
    let (numerator, denominator) = decimal(number).or_else(|| {
        ctx.error(
            "AUTHORING_LOWER_NUMBER",
            format!("invalid time literal '{}'", value.raw),
            value.span,
        );
        None
    })?;
    let unit_denominator = match unit {
        "s" => 1,
        "ms" => 1_000,
        "us" => 1_000_000,
        _ => {
            ctx.error(
                "AUTHORING_LOWER_TIME_UNIT",
                format!("time requires s, ms, or us, found '{unit}'"),
                value.span,
            );
            return None;
        }
    };
    let scaled = numerator.checked_mul(i128::from(ctx.timescale))?;
    let divisor = denominator.checked_mul(unit_denominator)?;
    if scaled % divisor != 0 {
        ctx.error(
            "AUTHORING_LOWER_TIME_PRECISION",
            format!("'{}' is not exact at the project timebase", value.raw),
            value.span,
        );
        return None;
    }
    let ticks = i64::try_from(scaled / divisor).ok()?;
    RationalTime::new(ticks, ctx.timescale).ok()
}

pub fn range(ctx: &mut Context, at: &NumberLiteral, duration: &NumberLiteral) -> Option<TimeRange> {
    TimeRange::new(time(ctx, at)?, time(ctx, duration)?).ok()
}

pub fn milliseconds(ctx: &mut Context, value: &NumberLiteral) -> Option<f64> {
    let value = time(ctx, value)?;
    Some(value.value as f64 * 1_000.0 / f64::from(value.timescale))
}

pub fn scalar(ctx: &mut Context, value: &NumberLiteral, unit: &str) -> Option<f64> {
    number(ctx, value, &[unit], unit)
}

pub fn number(
    ctx: &mut Context,
    value: &NumberLiteral,
    units: &[&str],
    purpose: &str,
) -> Option<f64> {
    let (number, actual_unit) = split(&value.raw);
    if !units.contains(&actual_unit) {
        ctx.error(
            "AUTHORING_LOWER_UNIT",
            format!("{purpose} has unsupported unit '{actual_unit}'"),
            value.span,
        );
        return None;
    }
    number
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .or_else(|| {
            ctx.error(
                "AUTHORING_LOWER_NUMBER",
                format!("invalid number '{}'", value.raw),
                value.span,
            );
            None
        })
}

pub fn unitless(ctx: &mut Context, value: &NumberLiteral) -> Option<f64> {
    scalar(ctx, value, "")
}

pub fn scale(ctx: &mut Context, value: &NumberLiteral) -> Option<f64> {
    let (_, unit) = split(&value.raw);
    let parsed = number(ctx, value, &["", "%"], "scale")?;
    match unit {
        "" => Some(parsed),
        "%" => Some(parsed / 100.0),
        _ => None,
    }
}

pub fn integer_i32(ctx: &mut Context, value: &NumberLiteral, purpose: &str) -> Option<i32> {
    let parsed = integer(ctx, value, purpose)?;
    i32::try_from(parsed).ok().or_else(|| {
        ctx.error(
            "AUTHORING_LOWER_INTEGER_RANGE",
            format!("{purpose} must fit in a 32-bit signed integer"),
            value.span,
        );
        None
    })
}

pub fn integer_u32(ctx: &mut Context, value: &NumberLiteral, purpose: &str) -> Option<u32> {
    let parsed = integer(ctx, value, purpose)?;
    u32::try_from(parsed).ok().or_else(|| {
        ctx.error(
            "AUTHORING_LOWER_INTEGER_RANGE",
            format!("{purpose} must be an unsigned 32-bit integer"),
            value.span,
        );
        None
    })
}

fn integer(ctx: &mut Context, value: &NumberLiteral, purpose: &str) -> Option<i128> {
    let (number, unit) = split(&value.raw);
    let parsed = decimal(number);
    if unit.is_empty() && parsed.is_some_and(|(_, denominator)| denominator == 1) {
        return parsed.map(|(numerator, _)| numerator);
    }
    ctx.error(
        "AUTHORING_LOWER_INTEGER",
        format!("{purpose} must be a unitless integer"),
        value.span,
    );
    None
}

pub fn rational(value: f64, span: Span, ctx: &mut Context) -> Option<Rational> {
    let numerator = (value * 1_000_000.0).round();
    let parsed = if numerator.is_finite() {
        Rational::new(numerator as i64, 1_000_000).ok()
    } else {
        None
    };
    parsed.or_else(|| {
        ctx.error(
            "AUTHORING_LOWER_RATIONAL",
            "value cannot be represented as a rational",
            span,
        );
        None
    })
}

pub fn split(raw: &str) -> (&str, &str) {
    let boundary = raw
        .char_indices()
        .find(|(_, value)| value.is_ascii_alphabetic() || *value == '%')
        .map_or(raw.len(), |(index, _)| index);
    raw.split_at(boundary)
}

pub fn decimal(raw: &str) -> Option<(i128, i128)> {
    let negative = raw.starts_with('-');
    let unsigned = raw.strip_prefix(['-', '+']).unwrap_or(raw);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty() || !whole.bytes().all(|value| value.is_ascii_digit()) {
        return None;
    }
    if !fraction.bytes().all(|value| value.is_ascii_digit()) {
        return None;
    }
    let denominator = 10_i128.checked_pow(u32::try_from(fraction.len()).ok()?)?;
    let combined = format!("{whole}{fraction}").parse::<i128>().ok()?;
    Some((if negative { -combined } else { combined }, denominator))
}
