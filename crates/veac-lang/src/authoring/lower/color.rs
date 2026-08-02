use crate::authoring::{Span, Spanned};
use veac_ir::Color;

use super::context::Context;

pub(super) fn lower(ctx: &mut Context, value: &Spanned<String>) -> Option<Color> {
    let Some(digits) = value.value.strip_prefix('#') else {
        return invalid(ctx, value.span);
    };
    let (red, green, blue, alpha) = match digits.len() {
        6 => (
            component(&digits[0..2]),
            component(&digits[2..4]),
            component(&digits[4..6]),
            Some(u8::MAX),
        ),
        8 => (
            component(&digits[0..2]),
            component(&digits[2..4]),
            component(&digits[4..6]),
            component(&digits[6..8]),
        ),
        _ => return invalid(ctx, value.span),
    };
    Some(Color {
        red: red.or_else(|| invalid(ctx, value.span))?,
        green: green.or_else(|| invalid(ctx, value.span))?,
        blue: blue.or_else(|| invalid(ctx, value.span))?,
        alpha: alpha.or_else(|| invalid(ctx, value.span))?,
    })
}

fn component(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}

fn invalid<T>(ctx: &mut Context, span: Span) -> Option<T> {
    ctx.error(
        "AUTHORING_LOWER_COLOR",
        "color must be #RRGGBB or #RRGGBBAA",
        span,
    );
    None
}
