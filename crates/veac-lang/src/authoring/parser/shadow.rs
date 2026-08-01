use crate::authoring::{SemanticEntry, SemanticValue, ShadowDecl, Spanned};

use super::{semantic, Parser};

pub(super) fn parse(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &'static str,
) -> Option<ShadowDecl> {
    let mut block = semantic::nested(parser, entry, context)?;
    let color_entry = semantic::required(parser, &mut block, "color", context)?;
    let color = color(parser, &color_entry)?;
    let opacity = required_number(parser, &mut block, "opacity", context)?;
    let blur = required_number(parser, &mut block, "blur", context)?;
    let offset_entry = semantic::required(parser, &mut block, "offset", context)?;
    let offset = super::modifier_static::vector(parser, &offset_entry)?;
    semantic::finish(parser, block, context);
    Some(ShadowDecl {
        color,
        opacity,
        blur,
        offset,
    })
}

fn color(parser: &mut Parser, entry: &SemanticEntry) -> Option<Spanned<String>> {
    match entry.values.as_slice() {
        [SemanticValue::Color(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            semantic::shape(parser, entry, "shadow color must be one color".to_owned());
            None
        }
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
