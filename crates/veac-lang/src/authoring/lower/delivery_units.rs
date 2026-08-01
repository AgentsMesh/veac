use crate::authoring::NumberLiteral;

use super::{context::Context, value};

pub(super) fn bitrate_u32(
    ctx: &mut Context,
    literal: &NumberLiteral,
    purpose: &str,
) -> Option<u32> {
    u32::try_from(bitrate(ctx, literal, purpose)?)
        .ok()
        .or_else(|| {
            range_error(ctx, literal, purpose, "32-bit unsigned integer");
            None
        })
}

pub(super) fn bitrate(ctx: &mut Context, literal: &NumberLiteral, purpose: &str) -> Option<u64> {
    scaled(
        ctx,
        literal,
        purpose,
        &[("bps", 1), ("kbps", 1_000), ("mbps", 1_000_000)],
    )
}

pub(super) fn buffer_size(
    ctx: &mut Context,
    literal: &NumberLiteral,
    purpose: &str,
) -> Option<u64> {
    scaled(
        ctx,
        literal,
        purpose,
        &[("bit", 1), ("kbit", 1_000), ("mbit", 1_000_000)],
    )
}

pub(super) fn sample_rate(
    ctx: &mut Context,
    literal: &NumberLiteral,
    purpose: &str,
) -> Option<u32> {
    let parsed = scaled(ctx, literal, purpose, &[("hz", 1), ("khz", 1_000)])?;
    u32::try_from(parsed).ok().or_else(|| {
        range_error(ctx, literal, purpose, "32-bit unsigned integer");
        None
    })
}

pub(super) fn pixels(ctx: &mut Context, literal: &NumberLiteral, purpose: &str) -> Option<u32> {
    let parsed = scaled(ctx, literal, purpose, &[("px", 1)])?;
    u32::try_from(parsed).ok().or_else(|| {
        range_error(ctx, literal, purpose, "32-bit unsigned integer");
        None
    })
}

fn scaled(
    ctx: &mut Context,
    literal: &NumberLiteral,
    purpose: &str,
    units: &[(&str, i128)],
) -> Option<u64> {
    let (number, unit) = value::split(&literal.raw);
    let multiplier = units
        .iter()
        .find_map(|(candidate, value)| (*candidate == unit).then_some(*value));
    let parsed = value::decimal(number)
        .zip(multiplier)
        .and_then(|((numerator, denominator), multiplier)| {
            let scaled = numerator.checked_mul(multiplier)?;
            (scaled >= 0 && scaled % denominator == 0).then_some(scaled / denominator)
        })
        .and_then(|value| u64::try_from(value).ok());
    parsed.or_else(|| {
        let labels = units
            .iter()
            .map(|(unit, _)| *unit)
            .collect::<Vec<_>>()
            .join(", ");
        ctx.error(
            "AUTHORING_LOWER_DELIVERY_UNIT",
            format!("{purpose} requires an exact non-negative value in {labels}"),
            literal.span,
        );
        None
    })
}

fn range_error(ctx: &mut Context, value: &NumberLiteral, purpose: &str, range: &str) {
    ctx.error(
        "AUTHORING_LOWER_DELIVERY_RANGE",
        format!("{purpose} must fit in a {range}"),
        value.span,
    );
}
