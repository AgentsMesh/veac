use crate::authoring::{Identifier, SemanticBlock, SemanticEntry, SemanticValue};

use super::super::Parser;

pub(in crate::authoring::parser) fn tagged_block(
    parser: &mut Parser,
    entry: SemanticEntry,
    context: &str,
) -> Option<(Identifier, SemanticBlock)> {
    let tag = one_tag(parser, &entry, context)?;
    match entry.block {
        Some(block) => Some((tag, block)),
        None => {
            parser.error(
                "AUTHORING_RECIPE_BODY_REQUIRED",
                format!("{context} requires a body"),
                entry.span,
            );
            None
        }
    }
}

pub(in crate::authoring::parser) fn tagged_leaf(
    parser: &mut Parser,
    entry: SemanticEntry,
    context: &str,
) -> Option<Identifier> {
    let tag = one_tag(parser, &entry, context)?;
    if entry.block.is_some() {
        parser.error(
            "AUTHORING_RECIPE_BODY_FORBIDDEN",
            format!("{context} does not accept a body"),
            entry.span,
        );
        None
    } else {
        Some(tag)
    }
}

fn one_tag(parser: &mut Parser, entry: &SemanticEntry, context: &str) -> Option<Identifier> {
    match entry.values.as_slice() {
        [SemanticValue::Identifier(value)] => Some(value.clone()),
        _ => {
            parser.error(
                "AUTHORING_RECIPE_VARIANT",
                format!("{context} requires exactly one closed variant"),
                entry.span,
            );
            None
        }
    }
}
