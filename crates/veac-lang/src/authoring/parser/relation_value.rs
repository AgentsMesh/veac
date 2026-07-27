use crate::authoring::{
    Diagnostic, Identifier, NumberLiteral, SemanticBlock, SemanticEntry, SemanticValue, Span,
    Spanned,
};

use super::Parser;

pub(super) fn take_block(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &str,
    required: bool,
    context: &str,
) -> Option<SemanticBlock> {
    let entry = take_entry(parser, body, name, required, context)?;
    if !entry.values.is_empty() || entry.block.is_none() {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_FIELD_TYPE",
            format!("{context}.{name} must be a block"),
            entry.span,
        ));
        return None;
    }
    entry.block
}

pub(super) fn take_entry(
    parser: &mut Parser,
    body: &mut SemanticBlock,
    name: &str,
    required: bool,
    context: &str,
) -> Option<SemanticEntry> {
    let Some(position) = body
        .entries
        .iter()
        .position(|entry| entry.name.value == name)
    else {
        if required {
            parser.diagnostic(Diagnostic::new(
                "AUTHORING_MISSING_FIELD",
                format!("{context} requires {name}"),
                body.span,
            ));
        }
        return None;
    };
    let entry = body.entries.remove(position);
    while let Some(position) = body
        .entries
        .iter()
        .position(|candidate| candidate.name.value == name)
    {
        let duplicate = body.entries.remove(position);
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_DUPLICATE_FIELD",
            format!("{context}.{name} may appear only once"),
            duplicate.span,
        ));
    }
    Some(entry)
}

pub(super) fn word(parser: &mut Parser, entry: SemanticEntry, label: &str) -> Option<Identifier> {
    match single_value(parser, entry, label)? {
        SemanticValue::Identifier(value) => Some(value),
        value => {
            parser.diagnostic(Diagnostic::new(
                "AUTHORING_FIELD_TYPE",
                format!("{label} must be a word"),
                value_span(&value),
            ));
            None
        }
    }
}

pub(super) fn number(
    parser: &mut Parser,
    entry: SemanticEntry,
    label: &str,
) -> Option<NumberLiteral> {
    match single_value(parser, entry, label)? {
        SemanticValue::Number(value) => Some(value),
        value => {
            parser.diagnostic(Diagnostic::new(
                "AUTHORING_FIELD_TYPE",
                format!("{label} must be a number"),
                value_span(&value),
            ));
            None
        }
    }
}

pub(super) fn boolean(
    parser: &mut Parser,
    entry: SemanticEntry,
    label: &str,
) -> Option<Spanned<bool>> {
    match single_value(parser, entry, label)? {
        SemanticValue::Boolean(value) => Some(value),
        value => {
            parser.diagnostic(Diagnostic::new(
                "AUTHORING_FIELD_TYPE",
                format!("{label} must be true or false"),
                value_span(&value),
            ));
            None
        }
    }
}

pub(super) fn finish(parser: &mut Parser, body: SemanticBlock, context: &str) {
    for entry in body.entries {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_UNKNOWN_FIELD",
            format!("unknown field {} in {context}", entry.name.value),
            entry.span,
        ));
    }
}

pub(super) fn unknown(parser: &mut Parser, code: &'static str, label: &str, value: Identifier) {
    parser.diagnostic(Diagnostic::new(
        code,
        format!("unknown {label} {}", value.value),
        value.span,
    ));
}

fn single_value(parser: &mut Parser, entry: SemanticEntry, label: &str) -> Option<SemanticValue> {
    if entry.block.is_some() || entry.values.len() != 1 {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_FIELD_TYPE",
            format!("{label} requires exactly one value"),
            entry.span,
        ));
        return None;
    }
    entry.values.into_iter().next()
}

fn value_span(value: &SemanticValue) -> Span {
    match value {
        SemanticValue::Identifier(value) => value.span,
        SemanticValue::Number(value) => value.span,
        SemanticValue::String(value) => value.span,
        SemanticValue::Color(value) => value.span,
        SemanticValue::Boolean(value) => value.span,
    }
}
