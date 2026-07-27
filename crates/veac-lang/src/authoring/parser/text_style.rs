use crate::authoring::{SemanticEntry, TextStyleDecl};

use super::{semantic, text_decor, text_span, text_value, Parser};

pub(super) fn parse(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextStyleDecl> {
    let mut block = semantic::nested(parser, entry, "text style")?;
    let font = match semantic::take(parser, &mut block, "font") {
        Some(entry) => Some(text_value::font(parser, &entry)?),
        None => None,
    };
    let fallback_fonts = text_value::take_all(&mut block, "fallback-font")
        .iter()
        .filter_map(|entry| text_value::font(parser, entry))
        .collect();
    let size = number(parser, &mut block, "size")?;
    let weight = enum_field(parser, &mut block, "weight", font_weight)?;
    let font_style = enum_field(parser, &mut block, "font-style", font_style)?;
    let fill = match semantic::take(parser, &mut block, "fill") {
        Some(entry) => Some(text_value::color(parser, &entry)?),
        None => None,
    };
    let tracking = number(parser, &mut block, "tracking")?;
    let line_height = number(parser, &mut block, "line-height")?;
    let background = nested(parser, &mut block, "background", text_decor::background)?;
    let outline = nested(parser, &mut block, "outline", text_decor::outline)?;
    let shadow = nested(parser, &mut block, "shadow", text_decor::shadow)?;
    let spans = text_value::take_all(&mut block, "span")
        .iter()
        .filter_map(|entry| text_span::parse(parser, entry))
        .collect();
    semantic::finish(parser, block, "text style");
    Some(TextStyleDecl {
        font,
        fallback_fonts,
        size,
        weight,
        font_style,
        fill,
        tracking,
        line_height,
        background,
        outline,
        shadow,
        spans,
    })
}

fn number(
    parser: &mut Parser,
    block: &mut crate::authoring::SemanticBlock,
    name: &'static str,
) -> Option<Option<crate::authoring::NumberLiteral>> {
    match semantic::take(parser, block, name) {
        Some(entry) => semantic::number(parser, &entry, name).map(Some),
        None => Some(None),
    }
}

fn enum_field<T>(
    parser: &mut Parser,
    block: &mut crate::authoring::SemanticBlock,
    name: &'static str,
    parse: fn(&str) -> Option<T>,
) -> Option<Option<crate::authoring::Spanned<T>>> {
    match semantic::take(parser, block, name) {
        Some(entry) => text_value::enum_value(parser, &entry, name, parse).map(Some),
        None => Some(None),
    }
}

pub(super) fn font_weight(value: &str) -> Option<veac_ir::FontWeight> {
    Some(match value {
        "thin" => veac_ir::FontWeight::Thin,
        "extra-light" => veac_ir::FontWeight::ExtraLight,
        "light" => veac_ir::FontWeight::Light,
        "normal" => veac_ir::FontWeight::Normal,
        "medium" => veac_ir::FontWeight::Medium,
        "semi-bold" => veac_ir::FontWeight::SemiBold,
        "bold" => veac_ir::FontWeight::Bold,
        "extra-bold" => veac_ir::FontWeight::ExtraBold,
        "black" => veac_ir::FontWeight::Black,
        _ => return None,
    })
}

pub(super) fn font_style(value: &str) -> Option<veac_ir::FontStyle> {
    Some(match value {
        "normal" => veac_ir::FontStyle::Normal,
        "italic" => veac_ir::FontStyle::Italic,
        "oblique" => veac_ir::FontStyle::Oblique,
        _ => return None,
    })
}

fn nested<T>(
    parser: &mut Parser,
    block: &mut crate::authoring::SemanticBlock,
    name: &'static str,
    parse: fn(&mut Parser, &SemanticEntry) -> Option<T>,
) -> Option<Option<T>> {
    match semantic::take(parser, block, name) {
        Some(entry) => parse(parser, &entry).map(Some),
        None => Some(None),
    }
}
