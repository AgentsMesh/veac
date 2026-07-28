use std::fmt::Write;

use veac_plan::canonical::{Color, FontStyle, FontWeight, Shadow};
use veac_plan::ResolvedTextStyle;

use super::escape::ass_name;
use super::model::AnimatedPiece;
use crate::emitter::time;

pub(super) fn piece(
    piece: &AnimatedPiece,
    opacity: f64,
    fill: Option<Color>,
    decoration: &ResolvedTextStyle,
) -> String {
    piece_style(&piece.style, opacity, fill, Some(decoration))
}

pub(super) fn glyph(
    style: &super::model::GlyphStyle,
    opacity: f64,
    fill: Option<Color>,
    decoration: &ResolvedTextStyle,
) -> String {
    piece_style(style, opacity, fill, Some(decoration))
}

pub(super) fn shadow_piece(piece: &AnimatedPiece, opacity: f64, shadow: &Shadow) -> String {
    piece_style(
        &piece.style,
        opacity * shadow.opacity,
        Some(shadow.color),
        None,
    )
}

pub(super) fn shadow_glyph(
    style: &super::model::GlyphStyle,
    opacity: f64,
    shadow: &Shadow,
) -> String {
    piece_style(style, opacity * shadow.opacity, Some(shadow.color), None)
}

fn piece_style(
    style: &super::model::GlyphStyle,
    opacity: f64,
    fill: Option<Color>,
    decoration: Option<&ResolvedTextStyle>,
) -> String {
    let fill = fill.unwrap_or(style.color);
    let mut tags = format!(
        "{{\\fn{}\\fs{}\\b{}\\i{}\\fsp{}\\1c{}\\1a{}}}",
        ass_name(&style.font_name),
        time::number(style.size),
        weight(style.weight),
        usize::from(style.font_style != FontStyle::Normal),
        time::number(style.tracking),
        color(fill),
        alpha(fill, opacity),
    );
    tags.pop();
    if let Some(outline) = decoration.and_then(|value| value.outline.as_ref()) {
        let _ = write!(tags, "\\3a{}", alpha(outline.color, opacity));
    }
    tags.push('}');
    tags
}

pub(super) fn fill_decoration(style: &ResolvedTextStyle) -> String {
    let mut tags = String::from("\\shad0\\blur0");
    if let Some(outline) = &style.outline {
        let _ = write!(
            tags,
            "\\bord{}\\3c{}\\3a{}",
            time::number(outline.width_pixels),
            color(outline.color),
            alpha(outline.color, 1.0)
        );
    } else {
        tags.push_str("\\bord0");
    }
    tags
}

pub(super) fn shadow_decoration(shadow: &Shadow) -> String {
    format!("\\bord0\\shad0\\blur{}", time::number(shadow.blur_pixels))
}

pub(super) fn color(value: Color) -> String {
    format!("&H{:02X}{:02X}{:02X}&", value.blue, value.green, value.red)
}

pub(super) fn alpha(value: Color, opacity: f64) -> String {
    let visible = f64::from(value.alpha) / 255.0 * opacity.clamp(0.0, 1.0);
    format!("&H{:02X}&", ((1.0 - visible) * 255.0).round() as u8)
}

pub(super) fn text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('\n', "\\N")
        .replace('\r', "")
}

fn weight(value: FontWeight) -> u16 {
    100 * (value as u16 + 1)
}
