use crate::authoring::{SemanticEntry, SemanticValue, Spanned, TextFontDecl};

use super::{semantic, Parser};

pub(super) fn string(parser: &mut Parser, entry: &SemanticEntry) -> Option<Spanned<String>> {
    match entry.values.as_slice() {
        [SemanticValue::String(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            semantic::shape(parser, entry, "field expects one string".to_owned());
            None
        }
    }
}

pub(super) fn color(parser: &mut Parser, entry: &SemanticEntry) -> Option<Spanned<String>> {
    match entry.values.as_slice() {
        [SemanticValue::Color(value)] if entry.block.is_none() => Some(value.clone()),
        _ => {
            semantic::shape(parser, entry, "field expects one color".to_owned());
            None
        }
    }
}

pub(super) fn font(parser: &mut Parser, entry: &SemanticEntry) -> Option<TextFontDecl> {
    match entry.values.as_slice() {
        [SemanticValue::Identifier(kind), SemanticValue::String(value)]
            if kind.value == "family" && entry.block.is_none() =>
        {
            Some(TextFontDecl::Family(value.clone()))
        }
        [SemanticValue::Identifier(kind), SemanticValue::Identifier(value)]
            if kind.value == "resource" && entry.block.is_none() =>
        {
            Some(TextFontDecl::Resource(value.clone()))
        }
        _ => invalid(
            parser,
            entry,
            "font expects family \"name\" or resource <id>",
        ),
    }
}

pub(super) fn enum_value<T>(
    parser: &mut Parser,
    entry: &SemanticEntry,
    context: &str,
    parse: impl FnOnce(&str) -> Option<T>,
) -> Option<Spanned<T>> {
    let [SemanticValue::Identifier(value)] = entry.values.as_slice() else {
        semantic::shape(parser, entry, format!("{context} expects one value"));
        return None;
    };
    parse(&value.value)
        .map(|parsed| Spanned {
            value: parsed,
            span: value.span,
        })
        .or_else(|| {
            invalid(
                parser,
                entry,
                &format!("unknown {context} '{}'", value.value),
            )
        })
}

pub(super) fn take_all(
    block: &mut crate::authoring::SemanticBlock,
    name: &str,
) -> Vec<SemanticEntry> {
    let (matching, rest): (Vec<_>, Vec<_>) = block
        .entries
        .drain(..)
        .partition(|entry| entry.name.value == name);
    block.entries = rest;
    matching
}

fn invalid<T>(parser: &mut Parser, entry: &SemanticEntry, message: &str) -> Option<T> {
    parser.error("AUTHORING_TEXT_FIELD", message.to_owned(), entry.span);
    None
}
