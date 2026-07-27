use crate::authoring::{CaptionSourceDecl, TextFontDecl, TextSourceDecl, TextStyleDecl};

use super::text_animation::animation;
use super::text_decor::{background, outline, shadow, span};
use super::text_layout::layout;
use super::value::quoted;
use super::writer::Writer;

pub(super) fn source(writer: &mut Writer, value: &TextSourceDecl) {
    writer.block("source text", |writer| {
        body(writer, value, None);
    });
}

pub(super) fn caption_source(writer: &mut Writer, value: &CaptionSourceDecl) {
    writer.block("source caption", |writer| {
        body(writer, &value.text, value.speaker.as_ref());
    });
}

fn body(
    writer: &mut Writer,
    value: &TextSourceDecl,
    speaker: Option<&crate::authoring::Spanned<String>>,
) {
    writer.line(format!("content {};", quoted(&value.content.value)));
    if let Some(value) = speaker {
        writer.line(format!("speaker {};", quoted(&value.value)));
    }
    style(writer, &value.style);
    layout(writer, &value.layout);
    if let Some(value) = &value.animation {
        animation(writer, value);
    }
}

fn style(writer: &mut Writer, value: &TextStyleDecl) {
    writer.block("style", |writer| {
        if let Some(value) = &value.font {
            font(writer, "font", value);
        }
        for value in &value.fallback_fonts {
            font(writer, "fallback-font", value);
        }
        number(writer, "size", value.size.as_ref());
        if let Some(value) = &value.weight {
            writer.line(format!(
                "weight {};",
                super::text_decor::weight(value.value)
            ));
        }
        if let Some(value) = &value.font_style {
            writer.line(format!(
                "font-style {};",
                super::text_decor::font_style(value.value)
            ));
        }
        if let Some(value) = &value.fill {
            writer.line(format!("fill {};", value.value));
        }
        number(writer, "tracking", value.tracking.as_ref());
        number(writer, "line-height", value.line_height.as_ref());
        if let Some(value) = &value.background {
            background(writer, value);
        }
        if let Some(value) = &value.outline {
            outline(writer, value);
        }
        if let Some(value) = &value.shadow {
            shadow(writer, value);
        }
        for value in &value.spans {
            span(writer, value);
        }
    });
}

pub(super) fn font(writer: &mut Writer, name: &str, value: &TextFontDecl) {
    match value {
        TextFontDecl::Family(value) => {
            writer.line(format!("{name} family {};", quoted(&value.value)));
        }
        TextFontDecl::Resource(value) => {
            writer.line(format!("{name} resource {};", value.value));
        }
    }
}

pub(super) fn number(
    writer: &mut Writer,
    name: &str,
    value: Option<&crate::authoring::NumberLiteral>,
) {
    if let Some(value) = value {
        writer.line(format!("{name} {};", value.raw));
    }
}
