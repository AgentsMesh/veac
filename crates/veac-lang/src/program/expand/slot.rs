mod source;

use std::collections::BTreeMap;

use crate::program::diagnostic::Diagnostic;
use crate::program::lexer;
use crate::program::model::{ComponentInterface, RawBlock, SlotDecl};
use crate::program::token::{Token, TokenKind};

use super::budget::{self, Replacements};
use super::{component, hygiene, text};

pub(super) fn expand_caller_fills(
    caller: &component::Caller<'_>,
    slots: &[SlotDecl],
    interface: &ComponentInterface,
    fills: &BTreeMap<String, RawBlock>,
    hygiene_path: Option<&hygiene::InstancePath>,
    registry: &mut hygiene::Registry,
    limit: usize,
) -> Result<BTreeMap<String, String>, Diagnostic> {
    validate_fill_names(caller.path, slots, interface, fills)?;
    let mut remaining = limit;
    let mut expanded_fills = BTreeMap::new();
    for slot in slots {
        let fill = &fills[&slot.name];
        let raw = &caller.source[fill.content_span.start..fill.content_span.end];
        let expanded = text::expand_with_limit(
            caller.path,
            raw,
            caller.presets,
            caller.values,
            0,
            remaining,
        )
        .map_err(|error| shift(error, fill.content_span.start))?;
        let hygienic = match hygiene_path {
            Some(path) => {
                hygiene::expand_with_limit(caller.path, expanded, path, registry, remaining)
                    .map_err(|error| shift(error, fill.content_span.start))?
            }
            None => reject_entry_locals(caller.path, expanded, fill.span)?,
        };
        let forwarded =
            inject_with_limit(caller.path, hygienic, caller.forwarded_slots, remaining)?;
        source::validate(caller.path, &forwarded, slot, fill.span)?;
        remaining = budget::consume(caller.path, remaining, forwarded.len())?;
        expanded_fills.insert(slot.name.clone(), forwarded);
    }
    Ok(expanded_fills)
}

pub(super) fn inject_with_limit(
    path: &str,
    source: String,
    fills: &BTreeMap<String, String>,
    limit: usize,
) -> Result<String, Diagnostic> {
    let tokens = lexer::lex(path, &source).map_err(|errors| errors[0].clone())?;
    let mut uses = Vec::new();
    for at in 0..tokens.len().saturating_sub(3) {
        let window = &tokens[at..at + 4];
        if !statement_start(&tokens, at)
            || window[0].word() != Some("source")
            || window[1].word() != Some("slot")
            || !matches!(window[3].kind, TokenKind::Semicolon)
        {
            continue;
        }
        let Some(name) = window[2].word() else {
            continue;
        };
        budget::ensure_replacement_available(path, uses.len())?;
        uses.push((
            window[0].span.start..window[3].span.end,
            name.to_owned(),
            window[2].span,
        ));
    }
    let spans = uses.iter().map(|(span, _, _)| span.clone());
    let mut replacements = Replacements::with_owned(path, source, spans, limit)?;
    for (_, name, span) in uses {
        let replacement = fills.get(&name).ok_or_else(|| {
            Diagnostic::new(
                "PROGRAM_SLOT_NOT_FILLED",
                path,
                format!("component body references unfilled slot `{name}`"),
                span,
            )
        })?;
        replacements.push_borrowed(replacement.trim())?;
    }
    replacements.finish()
}

fn reject_entry_locals(
    path: &str,
    source: String,
    span: crate::authoring::Span,
) -> Result<String, Diagnostic> {
    let tokens = lexer::lex(path, &source).map_err(|errors| errors[0].clone())?;
    if tokens
        .iter()
        .any(|token| matches!(&token.kind, TokenKind::LocalId(_)))
    {
        return Err(Diagnostic::new(
            "PROGRAM_FILL_LOCAL_SCOPE",
            path,
            "entry-scope component fills cannot reference component-local IDs",
            span,
        ));
    }
    Ok(source)
}

fn validate_fill_names(
    path: &str,
    slots: &[SlotDecl],
    interface: &ComponentInterface,
    fills: &BTreeMap<String, RawBlock>,
) -> Result<(), Diagnostic> {
    for slot in slots {
        fills.get(&slot.name).ok_or_else(|| {
            Diagnostic::new(
                "PROGRAM_SLOT_MISSING",
                path,
                format!("slot `{}` has no fill", slot.name),
                slot.span,
            )
        })?;
    }
    if let Some(name) = fills.keys().find(|name| !interface.has_slot(name)) {
        return Err(Diagnostic::new(
            "PROGRAM_SLOT_UNKNOWN",
            path,
            format!("slot `{name}` is not declared by the component"),
            fills[name].span,
        ));
    }
    Ok(())
}

fn shift(mut error: Diagnostic, base: usize) -> Diagnostic {
    error.span.start += base;
    error.span.end += base;
    error
}

fn statement_start(tokens: &[Token], at: usize) -> bool {
    at == 0
        || matches!(
            tokens[at - 1].kind,
            TokenKind::LeftBrace | TokenKind::RightBrace | TokenKind::Semicolon
        )
}

#[cfg(test)]
mod budget_tests;
