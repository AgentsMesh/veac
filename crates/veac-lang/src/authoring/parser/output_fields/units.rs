use crate::authoring::NumberLiteral;

use super::super::Parser;

pub(in crate::authoring::parser) fn bitrate(
    parser: &mut Parser,
    value: &NumberLiteral,
    name: &str,
) -> Option<u64> {
    scaled(
        parser,
        value,
        name,
        &[("bps", 1), ("kbps", 1_000), ("mbps", 1_000_000)],
    )
}

pub(in crate::authoring::parser) fn buffer_size(
    parser: &mut Parser,
    value: &NumberLiteral,
    name: &str,
) -> Option<u64> {
    scaled(
        parser,
        value,
        name,
        &[("bit", 1), ("kbit", 1_000), ("mbit", 1_000_000)],
    )
}

pub(in crate::authoring::parser) fn sample_rate(
    parser: &mut Parser,
    value: &NumberLiteral,
    name: &str,
) -> Option<u32> {
    let parsed = scaled(parser, value, name, &[("hz", 1), ("khz", 1_000)])?;
    u32::try_from(parsed).ok().or_else(|| {
        parser.error(
            "AUTHORING_OUTPUT_UNIT_RANGE",
            format!("{name} exceeds the supported range"),
            value.span,
        );
        None
    })
}

pub(in crate::authoring::parser) fn pixels(
    parser: &mut Parser,
    value: &NumberLiteral,
    name: &str,
) -> Option<u32> {
    let parsed = scaled(parser, value, name, &[("px", 1)])?;
    u32::try_from(parsed).ok().or_else(|| {
        parser.error(
            "AUTHORING_OUTPUT_UNIT_RANGE",
            format!("{name} exceeds the supported range"),
            value.span,
        );
        None
    })
}

fn scaled(
    parser: &mut Parser,
    value: &NumberLiteral,
    name: &str,
    units: &[(&str, u128)],
) -> Option<u64> {
    let boundary = value
        .raw
        .char_indices()
        .find(|(_, character)| character.is_ascii_alphabetic())
        .map_or(value.raw.len(), |(index, _)| index);
    let (number, unit) = value.raw.split_at(boundary);
    let multiplier = units
        .iter()
        .find_map(|(candidate, scale)| (*candidate == unit).then_some(*scale));
    let result = decimal(number)
        .zip(multiplier)
        .and_then(|((value, divisor), scale)| {
            let scaled = value.checked_mul(scale)?;
            (scaled % divisor == 0).then_some(scaled / divisor)
        });
    match result.and_then(|value| u64::try_from(value).ok()) {
        Some(value) => Some(value),
        None => {
            let labels = units
                .iter()
                .map(|(unit, _)| *unit)
                .collect::<Vec<_>>()
                .join(", ");
            parser.error(
                "AUTHORING_OUTPUT_UNIT",
                format!("{name} requires an exact non-negative value in {labels}"),
                value.span,
            );
            None
        }
    }
}

fn decimal(value: &str) -> Option<(u128, u128)> {
    let unsigned = value.strip_prefix('+').unwrap_or(value);
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let divisor = 10_u128.checked_pow(u32::try_from(fraction.len()).ok()?)?;
    let combined = format!("{whole}{fraction}").parse().ok()?;
    Some((combined, divisor))
}
