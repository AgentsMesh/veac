use veac_ir::{HueRange, LutInterpolation, ToneCurveInterpolation};

use super::context::Context;

pub(super) fn hue_range(
    ctx: &mut Context,
    value: &crate::authoring::Identifier,
) -> Option<HueRange> {
    enum_value(ctx, value, "hue range", |value| match value {
        "red" => Some(HueRange::Red),
        "yellow" => Some(HueRange::Yellow),
        "green" => Some(HueRange::Green),
        "cyan" => Some(HueRange::Cyan),
        "blue" => Some(HueRange::Blue),
        "magenta" => Some(HueRange::Magenta),
        _ => None,
    })
}

pub(super) fn curve_interpolation(
    ctx: &mut Context,
    value: &crate::authoring::Identifier,
) -> Option<ToneCurveInterpolation> {
    enum_value(ctx, value, "curve interpolation", |value| match value {
        "natural" => Some(ToneCurveInterpolation::Natural),
        "monotonic" => Some(ToneCurveInterpolation::Monotonic),
        _ => None,
    })
}

pub(super) fn lut_interpolation(
    ctx: &mut Context,
    value: &crate::authoring::Identifier,
) -> Option<LutInterpolation> {
    enum_value(ctx, value, "LUT interpolation", |value| match value {
        "nearest" => Some(LutInterpolation::Nearest),
        "linear" => Some(LutInterpolation::Linear),
        "cosine" => Some(LutInterpolation::Cosine),
        "cubic" => Some(LutInterpolation::Cubic),
        "spline" => Some(LutInterpolation::Spline),
        "trilinear" => Some(LutInterpolation::Trilinear),
        "tetrahedral" => Some(LutInterpolation::Tetrahedral),
        "pyramid" => Some(LutInterpolation::Pyramid),
        "prism" => Some(LutInterpolation::Prism),
        _ => None,
    })
}

fn enum_value<T>(
    ctx: &mut Context,
    value: &crate::authoring::Identifier,
    name: &str,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Option<T> {
    parse(&value.value).or_else(|| {
        ctx.error(
            "AUTHORING_LOWER_COLOR_ENUM",
            format!("unknown {name} '{}'", value.value),
            value.span,
        );
        None
    })
}
