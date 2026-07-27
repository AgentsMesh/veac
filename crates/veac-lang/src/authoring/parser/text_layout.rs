use crate::authoring::{SemanticEntry, TextLayoutDecl, TextPathDecl, VectorDecl};
use veac_ir::{
    HorizontalTextAlignment, TextOrientation, TextOverflow, TextPathAlignment, TextWrap,
    TextWritingMode, VerticalTextAlignment,
};

use super::{semantic, text_value, Parser};

pub(super) fn parse(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextLayoutDecl> {
    let mut block = semantic::nested(parser, entry, "text layout")?;
    let box_width = number(parser, &mut block, "box-width")?;
    let box_height = number(parser, &mut block, "box-height")?;
    let wrap = enum_field(parser, &mut block, "wrap", |value| match value {
        "none" => Some(TextWrap::None),
        "word" => Some(TextWrap::Word),
        "character" => Some(TextWrap::Character),
        _ => None,
    })?;
    let overflow = enum_field(parser, &mut block, "overflow", |value| match value {
        "visible" => Some(TextOverflow::Visible),
        "clip" => Some(TextOverflow::Clip),
        "ellipsis" => Some(TextOverflow::Ellipsis),
        _ => None,
    })?;
    let horizontal_alignment =
        enum_field(
            parser,
            &mut block,
            "horizontal-align",
            |value| match value {
                "left" => Some(HorizontalTextAlignment::Left),
                "center" => Some(HorizontalTextAlignment::Center),
                "right" => Some(HorizontalTextAlignment::Right),
                _ => None,
            },
        )?;
    let vertical_alignment =
        enum_field(parser, &mut block, "vertical-align", |value| match value {
            "top" => Some(VerticalTextAlignment::Top),
            "middle" => Some(VerticalTextAlignment::Middle),
            "bottom" => Some(VerticalTextAlignment::Bottom),
            _ => None,
        })?;
    let writing_mode = enum_field(parser, &mut block, "writing-mode", |value| match value {
        "horizontal-tb" => Some(TextWritingMode::HorizontalTb),
        "vertical-rl" => Some(TextWritingMode::VerticalRl),
        "vertical-lr" => Some(TextWritingMode::VerticalLr),
        _ => None,
    })?;
    let orientation = enum_field(parser, &mut block, "orientation", |value| match value {
        "upright" => Some(TextOrientation::Upright),
        "sideways" => Some(TextOrientation::Sideways),
        "mixed" => Some(TextOrientation::Mixed),
        _ => None,
    })?;
    let path = match semantic::take(parser, &mut block, "path") {
        Some(entry) => Some(path(parser, &entry)?),
        None => None,
    };
    semantic::finish(parser, block, "text layout");
    Some(TextLayoutDecl {
        box_width,
        box_height,
        wrap,
        overflow,
        horizontal_alignment,
        vertical_alignment,
        writing_mode,
        orientation,
        path,
    })
}

fn path(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextPathDecl> {
    let mut block = semantic::nested(parser, entry, "text path")?;
    let points = text_value::take_all(&mut block, "point")
        .iter()
        .filter_map(|entry| point(parser, entry))
        .collect::<Vec<_>>();
    if points.len() < 2 {
        parser.error(
            "AUTHORING_TEXT_PATH_POINTS",
            "text path requires at least two points".to_owned(),
            entry.span,
        );
    }
    let start_offset = required_number(parser, &mut block, "start-offset", "text path")?;
    let reverse_entry = semantic::required(parser, &mut block, "reverse", "text path")?;
    let reverse = semantic::boolean(parser, &reverse_entry, "text path reverse")?;
    let align_entry = semantic::required(parser, &mut block, "align", "text path")?;
    let align = text_value::enum_value(
        parser,
        &align_entry,
        "text path align",
        |value| match value {
            "start" => Some(TextPathAlignment::Start),
            "center" => Some(TextPathAlignment::Center),
            "end" => Some(TextPathAlignment::End),
            _ => None,
        },
    )?;
    semantic::finish(parser, block, "text path");
    Some(TextPathDecl {
        points,
        start_offset,
        reverse,
        align,
        span: entry.span,
    })
}

fn point(parser: &mut Parser, entry: &SemanticEntry) -> Option<VectorDecl> {
    let mut block = semantic::nested(parser, entry, "text path point")?;
    let x = required_number(parser, &mut block, "x", "text path point")?;
    let y = required_number(parser, &mut block, "y", "text path point")?;
    semantic::finish(parser, block, "text path point");
    Some(VectorDecl {
        x,
        y,
        span: entry.span,
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

fn required_number(
    parser: &mut Parser,
    block: &mut crate::authoring::SemanticBlock,
    name: &'static str,
    context: &'static str,
) -> Option<crate::authoring::NumberLiteral> {
    let entry = semantic::required(parser, block, name, context)?;
    semantic::number(parser, &entry, name)
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
