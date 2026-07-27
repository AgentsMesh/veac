use crate::authoring::{SemanticEntry, TextSpanDecl};

use super::{semantic, text_value, Parser};

pub(super) fn parse(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextSpanDecl> {
    let mut block = semantic::nested(parser, entry, "text span")?;
    let start = required_number(parser, &mut block, "start")?;
    let end = required_number(parser, &mut block, "end")?;
    let font = match semantic::take(parser, &mut block, "font") {
        Some(entry) => Some(text_value::font(parser, &entry)?),
        None => None,
    };
    let size = number(parser, &mut block, "size")?;
    let weight = match semantic::take(parser, &mut block, "weight") {
        Some(entry) => Some(super::text_value::enum_value(
            parser,
            &entry,
            "font weight",
            super::text_style::font_weight,
        )?),
        None => None,
    };
    let font_style = match semantic::take(parser, &mut block, "font-style") {
        Some(entry) => Some(super::text_value::enum_value(
            parser,
            &entry,
            "font style",
            super::text_style::font_style,
        )?),
        None => None,
    };
    let fill = match semantic::take(parser, &mut block, "fill") {
        Some(entry) => Some(text_value::color(parser, &entry)?),
        None => None,
    };
    semantic::finish(parser, block, "text span");
    Some(TextSpanDecl {
        start,
        end,
        font,
        size,
        weight,
        font_style,
        fill,
        span: entry.span,
    })
}

fn required_number(
    parser: &mut Parser,
    block: &mut crate::authoring::SemanticBlock,
    name: &'static str,
) -> Option<crate::authoring::NumberLiteral> {
    match number(parser, block, name)? {
        Some(value) => Some(value),
        None => {
            parser.error(
                "AUTHORING_MISSING_FIELD",
                format!("text span requires {name}"),
                block.span,
            );
            None
        }
    }
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
