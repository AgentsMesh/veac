use crate::authoring::{
    Diagnostic, Identifier, ItemEndpoint, SemanticEntry, SemanticValue, SignalEndpoint,
};

use super::Parser;

pub(super) fn item(parser: &mut Parser, entry: SemanticEntry, label: &str) -> Option<ItemEndpoint> {
    let (kind, id) = reference(parser, entry, label)?;
    if kind.value != "item" {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_RELATION_ENDPOINT_TYPE",
            format!("{label} requires an item endpoint"),
            kind.span,
        ));
        return None;
    }
    Some(ItemEndpoint { id })
}

pub(super) fn signal(
    parser: &mut Parser,
    entry: SemanticEntry,
    label: &str,
) -> Option<SignalEndpoint> {
    let (kind, id) = reference(parser, entry, label)?;
    match kind.value.as_str() {
        "track" => Some(SignalEndpoint::Track { id }),
        "bus" => Some(SignalEndpoint::Bus { id }),
        _ => {
            parser.diagnostic(Diagnostic::new(
                "AUTHORING_RELATION_ENDPOINT_TYPE",
                format!("{label} requires a track or bus endpoint"),
                kind.span,
            ));
            None
        }
    }
}

pub(super) fn member(
    parser: &mut Parser,
    entry: SemanticEntry,
    label: &str,
) -> Option<ItemEndpoint> {
    if entry.name.value != "item" || entry.block.is_some() || entry.values.len() != 1 {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_RELATION_ENDPOINT_TYPE",
            format!("{label} members must use `item <id>;`"),
            entry.span,
        ));
        return None;
    }
    match entry.values.into_iter().next() {
        Some(SemanticValue::Identifier(id)) => Some(ItemEndpoint { id }),
        _ => {
            parser.diagnostic(Diagnostic::new(
                "AUTHORING_FIELD_TYPE",
                format!("{label} item id must be a word"),
                entry.span,
            ));
            None
        }
    }
}

fn reference(
    parser: &mut Parser,
    entry: SemanticEntry,
    label: &str,
) -> Option<(Identifier, Identifier)> {
    if entry.block.is_some() || entry.values.len() != 2 {
        parser.diagnostic(Diagnostic::new(
            "AUTHORING_FIELD_TYPE",
            format!("{label} requires a typed reference"),
            entry.span,
        ));
        return None;
    }
    let mut values = entry.values.into_iter();
    match (values.next(), values.next()) {
        (Some(SemanticValue::Identifier(kind)), Some(SemanticValue::Identifier(id))) => {
            Some((kind, id))
        }
        _ => {
            parser.diagnostic(Diagnostic::new(
                "AUTHORING_FIELD_TYPE",
                format!("{label} requires word kind and id"),
                entry.span,
            ));
            None
        }
    }
}
