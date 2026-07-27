use std::fmt::Write;

use veac_plan::canonical::{Color, FontStyle, FontWeight};
use veac_plan::ResolvedTextStyle;

use super::super::failure::Failure;
use crate::emitter::time;

#[derive(Clone, PartialEq)]
pub(super) struct Style {
    pub font: String,
    pub size: f64,
    pub color: Color,
    pub weight: FontWeight,
    pub italic: bool,
    pub tracking: f64,
    pub outline: Option<(Color, f64)>,
    pub shadow: Option<(Color, f64, f64, f64, f64)>,
    pub alignment: u8,
}

impl Style {
    pub fn resolve(value: &ResolvedTextStyle, font: String) -> Result<Self, Failure> {
        super::validate::style(value)?;
        Ok(Self {
            font,
            size: value.size_pixels,
            color: value.color,
            weight: value.font_weight,
            italic: value.font_style == FontStyle::Italic,
            tracking: value.tracking_pixels,
            outline: value
                .outline
                .as_ref()
                .map(|item| (item.color, item.width_pixels)),
            shadow: value.shadow.as_ref().map(|item| {
                (
                    item.color,
                    item.opacity,
                    item.blur_pixels,
                    item.offset.x,
                    item.offset.y,
                )
            }),
            alignment: alignment(value),
        })
    }

    pub fn write_line(&self, output: &mut String, name: &str) {
        let outline = self
            .outline
            .map_or("&HFF000000".to_owned(), |value| ass_color(value.0, 1.0));
        let back = self
            .shadow
            .map_or("&HFF000000".to_owned(), |value| ass_color(value.0, value.1));
        let _ = writeln!(
            output,
            "Style: {name},{},{},{},{},{outline},{back},{},{},0,0,100,100,{},0,1,{},0,{},0,0,0,1",
            self.font,
            time::number(self.size),
            ass_color(self.color, 1.0),
            ass_color(self.color, 1.0),
            if weight(self.weight) >= 600 { -1 } else { 0 },
            if self.italic { -1 } else { 0 },
            time::number(self.tracking),
            time::number(self.outline.map_or(0.0, |value| value.1)),
            self.alignment,
        );
    }

    pub fn base_tags(&self) -> String {
        format!(
            "\\fn{}\\fs{}\\b{}\\i{}\\fsp{}\\1c{}\\1a{}",
            self.font,
            time::number(self.size),
            weight(self.weight),
            u8::from(self.italic),
            time::number(self.tracking),
            ass_rgb(self.color),
            ass_alpha(self.color, 1.0),
        )
    }
}

pub(super) fn ass_rgb(value: Color) -> String {
    format!("&H{:02X}{:02X}{:02X}&", value.blue, value.green, value.red)
}

pub(super) fn ass_alpha(value: Color, opacity: f64) -> String {
    let visible = f64::from(value.alpha) / 255.0 * opacity.clamp(0.0, 1.0);
    format!("&H{:02X}&", ((1.0 - visible) * 255.0).round() as u8)
}

fn ass_color(value: Color, opacity: f64) -> String {
    let alpha = ass_alpha(value, opacity);
    format!(
        "&H{}{:02X}{:02X}{:02X}",
        &alpha[2..4],
        value.blue,
        value.green,
        value.red
    )
}

pub(super) fn weight(value: FontWeight) -> u16 {
    100 * (value as u16 + 1)
}

fn alignment(value: &ResolvedTextStyle) -> u8 {
    use veac_plan::canonical::{HorizontalTextAlignment as H, VerticalTextAlignment as V};
    match (
        value.layout.horizontal_alignment,
        value.layout.vertical_alignment,
    ) {
        (H::Left, V::Bottom) => 1,
        (H::Center, V::Bottom) => 2,
        (H::Right, V::Bottom) => 3,
        (H::Left, V::Middle) => 4,
        (H::Center, V::Middle) => 5,
        (H::Right, V::Middle) => 6,
        (H::Left, V::Top) => 7,
        (H::Center, V::Top) => 8,
        (H::Right, V::Top) => 9,
    }
}
