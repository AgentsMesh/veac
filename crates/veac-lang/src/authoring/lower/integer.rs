use crate::authoring::NumberLiteral;

use super::context::Context;
use super::value;

pub(super) fn u64(ctx: &mut Context, value: &NumberLiteral, name: &str) -> Option<u64> {
    let parsed = exact(ctx, value, name)?;
    u64::try_from(parsed)
        .ok()
        .or_else(|| range(ctx, value, name, "u64"))
}

pub(super) fn u16(ctx: &mut Context, value: &NumberLiteral, name: &str) -> Option<u16> {
    let parsed = exact(ctx, value, name)?;
    u16::try_from(parsed)
        .ok()
        .or_else(|| range(ctx, value, name, "u16"))
}

fn exact(ctx: &mut Context, value: &NumberLiteral, name: &str) -> Option<i128> {
    let (number, unit) = value::split(&value.raw);
    let parsed = value::decimal(number);
    if unit.is_empty() && parsed.is_some_and(|(_, denominator)| denominator == 1) {
        return parsed.map(|(numerator, _)| numerator);
    }
    ctx.error(
        "AUTHORING_LOWER_INTEGER",
        format!("{name} must be a unitless integer"),
        value.span,
    );
    None
}

fn range<T>(ctx: &mut Context, value: &NumberLiteral, name: &str, kind: &str) -> Option<T> {
    ctx.error(
        "AUTHORING_LOWER_INTEGER_RANGE",
        format!("{name} must fit in {kind}"),
        value.span,
    );
    None
}
