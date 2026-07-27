use crate::authoring::{TextBackgroundDecl, TextOutlineDecl, TextShadowDecl, TextSpanDecl};
use veac_ir::{FontStyle, FontWeight};

use super::text::{font, number};
use super::writer::Writer;

pub(super) fn background(writer: &mut Writer, value: &TextBackgroundDecl) {
    writer.block("background", |writer| {
        writer.line(format!("color {};", value.color.value));
        writer.line(format!("padding {};", value.padding.raw));
    });
}

pub(super) fn outline(writer: &mut Writer, value: &TextOutlineDecl) {
    writer.block("outline", |writer| {
        writer.line(format!("color {};", value.color.value));
        writer.line(format!("width {};", value.width.raw));
    });
}

pub(super) fn shadow(writer: &mut Writer, value: &TextShadowDecl) {
    writer.block("shadow", |writer| {
        writer.line(format!("color {};", value.color.value));
        writer.line(format!("opacity {};", value.opacity.raw));
        writer.line(format!("blur {};", value.blur.raw));
        writer.block("offset", |writer| {
            writer.line(format!("x {};", value.offset.x.raw));
            writer.line(format!("y {};", value.offset.y.raw));
        });
    });
}

pub(super) fn span(writer: &mut Writer, value: &TextSpanDecl) {
    writer.block("span", |writer| {
        writer.line(format!("start {};", value.start.raw));
        writer.line(format!("end {};", value.end.raw));
        if let Some(value) = &value.font {
            font(writer, "font", value);
        }
        number(writer, "size", value.size.as_ref());
        if let Some(value) = &value.weight {
            writer.line(format!("weight {};", weight(value.value)));
        }
        if let Some(value) = &value.font_style {
            writer.line(format!("font-style {};", font_style(value.value)));
        }
        if let Some(value) = &value.fill {
            writer.line(format!("fill {};", value.value));
        }
    });
}

pub(super) fn weight(value: FontWeight) -> &'static str {
    match value {
        FontWeight::Thin => "thin",
        FontWeight::ExtraLight => "extra-light",
        FontWeight::Light => "light",
        FontWeight::Normal => "normal",
        FontWeight::Medium => "medium",
        FontWeight::SemiBold => "semi-bold",
        FontWeight::Bold => "bold",
        FontWeight::ExtraBold => "extra-bold",
        FontWeight::Black => "black",
    }
}

pub(super) fn font_style(value: FontStyle) -> &'static str {
    match value {
        FontStyle::Normal => "normal",
        FontStyle::Italic => "italic",
        FontStyle::Oblique => "oblique",
    }
}
