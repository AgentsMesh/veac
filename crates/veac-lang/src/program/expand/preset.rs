use std::borrow::Cow;
use std::collections::BTreeMap;

use super::budget::{self, Replacements};
use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::program::lexer;
use crate::program::model::{preset_key, preset_name_exists, PresetKey, PresetKind};
use crate::program::parser::kind;
use crate::program::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PresetUse {
    pub kind: PresetKind,
    pub name: String,
    pub span: Span,
}

pub(super) fn expand_source<'source, P: AsRef<str>>(
    path: &str,
    source: Cow<'source, str>,
    presets: &'source BTreeMap<PresetKey, P>,
    limit: usize,
) -> Result<Cow<'source, str>, Diagnostic> {
    let tokens = lexer::lex(path, &source).map_err(|errors| errors[0].clone())?;
    let uses = find_uses(path, &tokens)?;
    let spans = uses
        .iter()
        .map(|preset_use| preset_use.span.start..preset_use.span.end);
    let mut replacements = Replacements::with_source(path, source, spans, limit)?;
    for preset_use in &uses {
        let key = preset_key(preset_use.kind, &preset_use.name);
        let body = presets
            .get(&key)
            .ok_or_else(|| missing(path, presets, preset_use))?;
        replacements.push_borrowed(body.as_ref())?;
    }
    replacements.finish_cow()
}

pub(crate) fn uses(path: &str, source: &str) -> Result<Vec<PresetUse>, Diagnostic> {
    let tokens = lexer::lex(path, source).map_err(|errors| errors[0].clone())?;
    find_uses(path, &tokens)
}

fn find_uses(path: &str, tokens: &[Token]) -> Result<Vec<PresetUse>, Diagnostic> {
    let mut found = Vec::new();
    for at in 0..tokens.len().saturating_sub(3) {
        let window = &tokens[at..at + 4];
        if !statement_start(tokens, at)
            || window[0].word() != Some("use")
            || !matches!(window[3].kind, TokenKind::Semicolon)
        {
            continue;
        }
        let Some(kind_name) = window[1].word() else {
            continue;
        };
        let Some(name) = window[2].word() else {
            continue;
        };
        let Some(preset_kind) = parse_kind(kind_name) else {
            return Err(Diagnostic::new(
                "PROGRAM_PRESET_KIND",
                path,
                "unknown preset kind at use site",
                window[1].span,
            ));
        };
        budget::ensure_replacement_available(path, found.len())?;
        found.push(PresetUse {
            kind: preset_kind,
            name: name.to_owned(),
            span: window[0].span.join(window[3].span),
        });
    }
    Ok(found)
}

fn missing<P>(path: &str, presets: &BTreeMap<PresetKey, P>, preset_use: &PresetUse) -> Diagnostic {
    let kind_name = kind::preset_name(preset_use.kind);
    let wrong_kind = preset_name_exists(presets, &preset_use.name);
    if wrong_kind {
        Diagnostic::new(
            "PROGRAM_PRESET_KIND_MISMATCH",
            path,
            format!("preset `{}` is not a `{kind_name}` preset", preset_use.name),
            preset_use.span,
        )
    } else {
        Diagnostic::new(
            "PROGRAM_PRESET_NOT_FOUND",
            path,
            format!("preset `{kind_name} {}` was not found", preset_use.name),
            preset_use.span,
        )
    }
}

fn statement_start(tokens: &[Token], at: usize) -> bool {
    at == 0
        || matches!(
            tokens[at - 1].kind,
            TokenKind::LeftBrace | TokenKind::RightBrace | TokenKind::Semicolon
        )
}

fn parse_kind(value: &str) -> Option<PresetKind> {
    [
        PresetKind::TextStyle,
        PresetKind::TextLayout,
        PresetKind::ModifierStack,
        PresetKind::EffectPipeline,
        PresetKind::ColorPipeline,
        PresetKind::AudioProcessors,
        PresetKind::DeliveryProfile,
    ]
    .into_iter()
    .find(|candidate| kind::preset_name(*candidate) == value)
}

#[cfg(test)]
mod budget_tests;
