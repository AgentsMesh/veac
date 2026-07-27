use std::fmt::Write;

use veac_plan::canonical::{Color, FontStyle, FontWeight};
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
    piece_style(&piece.style, opacity, fill, decoration)
}

pub(super) fn glyph(
    style: &super::model::GlyphStyle,
    opacity: f64,
    fill: Option<Color>,
    decoration: &ResolvedTextStyle,
) -> String {
    piece_style(style, opacity, fill, decoration)
}

fn piece_style(
    style: &super::model::GlyphStyle,
    opacity: f64,
    fill: Option<Color>,
    decoration: &ResolvedTextStyle,
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
    if let Some(outline) = &decoration.outline {
        let _ = write!(tags, "\\3a{}", alpha(outline.color, opacity));
    }
    if let Some(shadow) = &decoration.shadow {
        let _ = write!(
            tags,
            "\\4a{}",
            alpha(shadow.color, shadow.opacity * opacity)
        );
    }
    tags.push('}');
    tags
}

pub(super) fn decoration(style: &ResolvedTextStyle) -> String {
    let mut tags = String::new();
    if let Some(outline) = &style.outline {
        let _ = write!(
            tags,
            "\\bord{}\\3c{}\\3a{}",
            time::number(outline.width_pixels),
            color(outline.color),
            alpha(outline.color, 1.0)
        );
    }
    if let Some(shadow) = &style.shadow {
        if shadow.blur_pixels > 0.0 {
            let _ = write!(tags, "\\blur{}", time::number(shadow.blur_pixels));
        }
        let _ = write!(
            tags,
            "\\xshad{}\\yshad{}\\4c{}\\4a{}",
            time::number(shadow.offset.x),
            time::number(shadow.offset.y),
            color(shadow.color),
            alpha(shadow.color, shadow.opacity)
        );
    }
    tags
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
