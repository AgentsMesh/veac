use crate::authoring::{SemanticBlock, SemanticEntry, SemanticValue, Spanned};

use super::{semantic, Parser};

pub(super) fn required(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<SemanticEntry> {
    semantic::required(parser, block, name, "annotation")
}

pub(super) fn take_all(block: &mut SemanticBlock, name: &str) -> Vec<SemanticEntry> {
    let mut matches = Vec::new();
    let mut rest = Vec::new();
    for entry in block.entries.drain(..) {
        if entry.name.value == name {
            matches.push(entry);
        } else {
            rest.push(entry);
        }
    }
    block.entries = rest;
    matches
}

pub(super) fn finish(parser: &mut Parser, block: SemanticBlock) {
    semantic::finish(parser, block, "annotation")
}

pub(super) fn string(parser: &mut Parser, entry: &SemanticEntry) -> Option<Spanned<String>> {
    match entry.values.as_slice() {
        [SemanticValue::String(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            semantic::shape(parser, entry, "field must be one string".to_owned());
            None
        }
    }
}

pub(super) fn color(parser: &mut Parser, entry: &SemanticEntry) -> Option<Spanned<String>> {
    match entry.values.as_slice() {
        [SemanticValue::Color(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            semantic::shape(parser, entry, "field must be one color".to_owned());
            None
        }
    }
}

pub(super) fn shape(parser: &mut Parser, entry: &SemanticEntry, expected: &str) {
    semantic::shape(parser, entry, format!("field expects {expected}"));
}

pub(super) fn header_block(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &str,
) -> Option<SemanticBlock> {
    entry.block.clone().or_else(|| {
        parser.error(
            "AUTHORING_FIELD_SHAPE",
            format!("{context} must have a block"),
            entry.span,
        );
        None
    })
}
