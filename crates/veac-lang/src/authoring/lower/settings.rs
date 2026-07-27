use crate::authoring::{Diagnostics, NumberLiteral, ProjectSettings, Span};
use veac_ir::Rational;

use super::context::Context;
use super::value::{decimal, split};

pub struct LoweredSettings {
    pub timescale: u32,
    pub frame_rate: Rational,
    pub width: u32,
    pub height: u32,
    pub sample_rate: u32,
}

pub fn lower(value: &ProjectSettings, span: Span) -> Result<LoweredSettings, Diagnostics> {
    let mut ctx = Context::new(1);
    let timescale = required(&mut ctx, value.timebase.as_ref(), "timebase", span)
        .and_then(|value| timescale(&mut ctx, value))
        .unwrap_or(1);
    ctx.timescale = timescale;
    let frame_rate = required(&mut ctx, value.frame_rate.as_ref(), "frame-rate", span)
        .and_then(|value| frame_rate(&mut ctx, value))
        .unwrap_or_else(|| Rational::new(30, 1).expect("valid fallback"));
    let (width, height) = value
        .canvas
        .as_ref()
        .and_then(|(width, height)| {
            Some((dimension(&mut ctx, width)?, dimension(&mut ctx, height)?))
        })
        .unwrap_or_else(|| {
            ctx.error(
                "AUTHORING_LOWER_REQUIRED",
                "project settings require canvas",
                span,
            );
            (1, 1)
        });
    let sample_rate = required(&mut ctx, value.sample_rate.as_ref(), "sample-rate", span)
        .and_then(|value| unsigned(&mut ctx, value, "hz"))
        .unwrap_or(48_000);
    if ctx.is_valid() {
        Ok(LoweredSettings {
            timescale,
            frame_rate,
            width,
            height,
            sample_rate,
        })
    } else {
        Err(ctx.diagnostics())
    }
}

fn required<'a, T>(
    ctx: &mut Context,
    value: Option<&'a T>,
    name: &str,
    span: Span,
) -> Option<&'a T> {
    if value.is_none() {
        ctx.error(
            "AUTHORING_LOWER_REQUIRED",
            format!("project settings require {name}"),
            span,
        );
    }
    value
}

fn timescale(ctx: &mut Context, value: &NumberLiteral) -> Option<u32> {
    let Some((numerator, denominator)) = value.raw.split_once('/') else {
        ctx.error(
            "AUTHORING_LOWER_TIMEBASE",
            "timebase must be 1/<positive-timescale>",
            value.span,
        );
        return None;
    };
    let parsed = numerator
        .parse::<u32>()
        .ok()
        .zip(denominator.parse::<u32>().ok());
    match parsed {
        Some((1, denominator)) if denominator > 0 => Some(denominator),
        _ => {
            ctx.error(
                "AUTHORING_LOWER_TIMEBASE",
                "timebase must be 1/<positive-timescale>",
                value.span,
            );
            None
        }
    }
}

fn frame_rate(ctx: &mut Context, value: &NumberLiteral) -> Option<Rational> {
    let (number, unit) = split(&value.raw);
    if unit != "fps" {
        ctx.error(
            "AUTHORING_LOWER_FRAME_RATE",
            "frame-rate requires fps",
            value.span,
        );
        return None;
    }
    let parsed = if let Some((left, right)) = number.split_once('/') {
        left.parse::<i64>().ok().zip(right.parse::<u32>().ok())
    } else {
        decimal(number)
            .and_then(|(left, right)| i64::try_from(left).ok().zip(u32::try_from(right).ok()))
    };
    parsed
        .and_then(|(numerator, denominator)| Rational::new(numerator, denominator).ok())
        .or_else(|| {
            ctx.error(
                "AUTHORING_LOWER_FRAME_RATE",
                "frame-rate must be a representable rational with a positive denominator",
                value.span,
            );
            None
        })
}

fn dimension(ctx: &mut Context, value: &NumberLiteral) -> Option<u32> {
    unsigned(ctx, value, "px")
}

fn unsigned(ctx: &mut Context, value: &NumberLiteral, unit: &str) -> Option<u32> {
    let (number, actual) = split(&value.raw);
    if actual != unit {
        ctx.error(
            "AUTHORING_LOWER_UNIT",
            format!("expected {unit}, found '{actual}'"),
            value.span,
        );
        return None;
    }
    number
        .parse::<u64>()
        .ok()
        .and_then(|value| u32::try_from(value).ok())
        .or_else(|| {
            ctx.error(
                "AUTHORING_LOWER_INTEGER_RANGE",
                format!("{unit} value must be an unsigned 32-bit integer"),
                value.span,
            );
            None
        })
}
