use super::Parser;
use crate::authoring::{
    Identifier, NumberLiteral, SemanticBlock, SemanticEntry, SemanticValue, Spanned,
};

pub(super) fn take(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
) -> Option<SemanticEntry> {
    let indexes: Vec<_> = block
        .entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| (entry.name.value == name).then_some(index))
        .collect();
    if indexes.len() > 1 {
        parser.duplicate(name, block.entries[indexes[1]].span);
    }
    indexes.first().map(|index| block.entries.remove(*index))
}

pub(super) fn required(
    parser: &mut Parser,
    block: &mut SemanticBlock,
    name: &'static str,
    context: &'static str,
) -> Option<SemanticEntry> {
    let value = take(parser, block, name);
    if value.is_none() {
        parser.error(
            "AUTHORING_REQUIRED_FIELD",
            format!("{context} requires {name}"),
            block.span,
        );
    }
    value
}

pub(super) fn finish(parser: &mut Parser, block: SemanticBlock, context: &'static str) {
    for entry in block.entries {
        parser.error(
            "AUTHORING_UNKNOWN_FIELD",
            format!("{context} does not support `{}`", entry.name.value),
            entry.name.span,
        );
    }
}

pub(super) fn number(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &'static str,
) -> Option<NumberLiteral> {
    match entry.values.as_slice() {
        [SemanticValue::Number(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            shape(parser, entry, format!("{context} must be one number"));
            None
        }
    }
}

pub(super) fn word(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &'static str,
) -> Option<Identifier> {
    match entry.values.as_slice() {
        [SemanticValue::Identifier(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            shape(parser, entry, format!("{context} must be one identifier"));
            None
        }
    }
}

pub(super) fn boolean(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &'static str,
) -> Option<Spanned<bool>> {
    match entry.values.as_slice() {
        [SemanticValue::Boolean(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            shape(parser, entry, format!("{context} must be true or false"));
            None
        }
    }
}

pub(super) fn nested(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &'static str,
) -> Option<SemanticBlock> {
    if entry.values.is_empty() {
        if let Some(block) = &entry.block {
            return Some(block.clone());
        }
    }
    shape(parser, entry, format!("{context} must be a block"));
    None
}

pub(super) fn shape(parser: &mut Parser, entry: &SemanticEntry, message: String) {
    parser.error("AUTHORING_FIELD_SHAPE", message, entry.span);
}
