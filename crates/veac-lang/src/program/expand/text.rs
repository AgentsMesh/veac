mod scanner;

use std::borrow::Cow;
use std::collections::BTreeMap;

use super::budget::{Replacements, MAX_EXPANDED_BYTES};
use super::preset;
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::{self, ValueLookup};
use crate::program::model::{PresetKey, PresetMap};

pub(super) fn expand(
    path: &str,
    source: &str,
    presets: &PresetMap,
    values: &dyn ValueLookup,
    depth: usize,
) -> Result<String, Diagnostic> {
    expand_with_limit(path, source, presets, values, depth, MAX_EXPANDED_BYTES)
}

pub(super) fn expand_with_limit<P: AsRef<str>>(
    path: &str,
    source: &str,
    presets: &BTreeMap<PresetKey, P>,
    values: &dyn ValueLookup,
    depth: usize,
    limit: usize,
) -> Result<String, Diagnostic> {
    if depth > 32 {
        return Err(error(
            path,
            "PROGRAM_EXPANSION_DEPTH",
            "expansion exceeds 32 levels",
        ));
    }
    let with_presets = preset::expand_source(path, Cow::Borrowed(source), presets, limit)?;
    expressions(path, with_presets, values, limit)
}

fn expressions(
    path: &str,
    source: Cow<'_, str>,
    values: &dyn ValueLookup,
    limit: usize,
) -> Result<String, Diagnostic> {
    let spans = scanner::expression_spans(path, &source)?;
    let mut replacements = Replacements::with_source(path, source, spans, limit)?;
    for index in 0..replacements.len() {
        let span = replacements.span(index);
        let value = evaluate(
            path,
            replacements.source(),
            values,
            span.start,
            span.end - 1,
        )?;
        replacements.push_owned(value)?;
    }
    replacements.finish()
}

fn evaluate(
    path: &str,
    source: &str,
    values: &dyn ValueLookup,
    start: usize,
    end: usize,
) -> Result<String, Diagnostic> {
    let value =
        expression::evaluate_with(source[start + 2..end].trim(), values).map_err(|cause| {
            Diagnostic::new(
                "PROGRAM_EXPRESSION",
                path,
                cause.to_string(),
                crate::authoring::Span {
                    start,
                    end: end + 1,
                },
            )
        })?;
    value.render_source_literal().map_err(|message| {
        Diagnostic::new(
            "PROGRAM_EXPRESSION_MATERIALIZATION",
            path,
            message,
            crate::authoring::Span {
                start,
                end: end + 1,
            },
        )
    })
}

fn error(path: &str, code: &'static str, message: &str) -> Diagnostic {
    Diagnostic::new(code, path, message, crate::authoring::Span::default())
}

#[cfg(test)]
mod budget_tests;
