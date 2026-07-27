use crate::authoring::{
    SemanticEntry, TextBackgroundDecl, TextOutlineDecl, TextShadowDecl, VectorDecl,
};

use super::{semantic, text_value, Parser};

pub(super) fn background(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextBackgroundDecl> {
    let mut block = semantic::nested(parser, entry, "text background")?;
    let color_entry = semantic::required(parser, &mut block, "color", "text background")?;
    let color = text_value::color(parser, &color_entry)?;
    let padding = required_number(parser, &mut block, "padding", "text background")?;
    semantic::finish(parser, block, "text background");
    Some(TextBackgroundDecl { color, padding })
}

pub(super) fn outline(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextOutlineDecl> {
    let mut block = semantic::nested(parser, entry, "text outline")?;
    let color_entry = semantic::required(parser, &mut block, "color", "text outline")?;
    let color = text_value::color(parser, &color_entry)?;
    let width = required_number(parser, &mut block, "width", "text outline")?;
    semantic::finish(parser, block, "text outline");
    Some(TextOutlineDecl { color, width })
}

pub(super) fn shadow(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextShadowDecl> {
    let mut block = semantic::nested(parser, entry, "text shadow")?;
    let color_entry = semantic::required(parser, &mut block, "color", "text shadow")?;
    let color = text_value::color(parser, &color_entry)?;
    let opacity = required_number(parser, &mut block, "opacity", "text shadow")?;
    let blur = required_number(parser, &mut block, "blur", "text shadow")?;
    let offset_entry = semantic::required(parser, &mut block, "offset", "text shadow")?;
    let offset = vector(parser, &offset_entry)?;
    semantic::finish(parser, block, "text shadow");
    Some(TextShadowDecl {
        color,
        opacity,
        blur,
        offset,
    })
}

fn vector(parser: &mut Parser, entry: &SemanticEntry) -> Option<VectorDecl> {
    let mut block = semantic::nested(parser, entry, "shadow offset")?;
    let x = required_number(parser, &mut block, "x", "shadow offset")?;
    let y = required_number(parser, &mut block, "y", "shadow offset")?;
    semantic::finish(parser, block, "shadow offset");
    Some(VectorDecl {
        x,
        y,
        span: entry.span,
    })
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
