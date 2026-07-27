use crate::authoring::{TextFontDecl, TextSpanDecl, TextStyleDecl};
use veac_ir::{FontRef, Shadow, TextBackground, TextOutline, TextSpan, TextStyle, Vec2};

use super::context::Context;
use super::{color, ids, value};

pub(super) fn lower(ctx: &mut Context, declaration: &TextStyleDecl) -> Option<TextStyle> {
    let default = TextStyle::default();
    Some(TextStyle {
        font: match &declaration.font {
            Some(value) => font(ctx, value)?,
            None => default.font,
        },
        fallback_fonts: declaration
            .fallback_fonts
            .iter()
            .map(|value| font(ctx, value))
            .collect::<Option<Vec<_>>>()?,
        font_weight: declaration
            .weight
            .as_ref()
            .map_or(default.font_weight, |value| value.value),
        font_style: declaration
            .font_style
            .as_ref()
            .map_or(default.font_style, |value| value.value),
        size_pixels: declaration
            .size
            .as_ref()
            .map_or(Some(default.size_pixels), |value| {
                value::scalar(ctx, value, "px")
            })?,
        color: match &declaration.fill {
            Some(value) => color::lower(ctx, value)?,
            None => default.color,
        },
        tracking_pixels: declaration
            .tracking
            .as_ref()
            .map_or(Some(default.tracking_pixels), |value| {
                value::scalar(ctx, value, "px")
            })?,
        line_height: declaration
            .line_height
            .as_ref()
            .map_or(Some(default.line_height), |value| {
                value::unitless(ctx, value)
            })?,
        layout: default.layout,
        path: None,
        background: match &declaration.background {
            Some(value) => Some(TextBackground {
                color: color::lower(ctx, &value.color)?,
                padding_pixels: value::scalar(ctx, &value.padding, "px")?,
            }),
            None => None,
        },
        outline: match &declaration.outline {
            Some(value) => Some(TextOutline {
                color: color::lower(ctx, &value.color)?,
                width_pixels: value::scalar(ctx, &value.width, "px")?,
            }),
            None => None,
        },
        shadow: match &declaration.shadow {
            Some(value) => Some(Shadow {
                blur_pixels: value::scalar(ctx, &value.blur, "px")?,
                opacity: value::scale(ctx, &value.opacity)?,
                offset: Vec2 {
                    x: value::scalar(ctx, &value.offset.x, "px")?,
                    y: value::scalar(ctx, &value.offset.y, "px")?,
                },
                color: color::lower(ctx, &value.color)?,
            }),
            None => None,
        },
        spans: declaration
            .spans
            .iter()
            .map(|value| span(ctx, value))
            .collect::<Option<Vec<_>>>()?,
        animation: None,
    })
}

fn span(ctx: &mut Context, value: &TextSpanDecl) -> Option<TextSpan> {
    Some(TextSpan {
        start: super::value::integer_u32(ctx, &value.start, "text span start")?,
        end: super::value::integer_u32(ctx, &value.end, "text span end")?,
        font: match &value.font {
            Some(value) => Some(font(ctx, value)?),
            None => None,
        },
        font_weight: value.weight.as_ref().map(|value| value.value),
        font_style: value.font_style.as_ref().map(|value| value.value),
        size_pixels: match &value.size {
            Some(value) => Some(super::value::scalar(ctx, value, "px")?),
            None => None,
        },
        color: match &value.fill {
            Some(value) => Some(color::lower(ctx, value)?),
            None => None,
        },
    })
}

fn font(ctx: &mut Context, value: &TextFontDecl) -> Option<FontRef> {
    Some(match value {
        TextFontDecl::Family(value) => FontRef::Family {
            family: value.value.clone(),
        },
        TextFontDecl::Resource(value) => FontRef::Material {
            material_id: ids::material(ctx, value)?,
        },
    })
}
