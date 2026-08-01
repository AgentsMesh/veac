use crate::authoring::{SemanticEntry, ShadowDecl, TextBackgroundDecl, TextOutlineDecl};

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

pub(super) fn shadow(parser: &mut Parser, entry: &SemanticEntry) -> Option<ShadowDecl> {
    super::shadow::parse(parser, entry, "text shadow")
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
