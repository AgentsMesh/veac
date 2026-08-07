use std::path::Path;

use super::diagnostic::{Diagnostic, Diagnostics};
use super::lexer;
use super::loader::{FileSystemLoader, LoadedSource, MemoryLoader, SourceLoader};
use crate::authoring::Span;

mod layout;
mod preservation;
mod trivia;
mod validation;

/// Validates and canonically formats one entry or standalone module from disk.
pub fn format_path(path: &Path) -> Result<String, Diagnostics> {
    let (loader, source) = FileSystemLoader::for_entry(path)
        .map_err(|message| Diagnostics::one(load_error(path, message)))?;
    format_source_with_loader(source, &loader)
}

/// Validates and canonically formats one import-free source unit.
pub fn format_source(source: &str) -> Result<String, Diagnostics> {
    format_source_with_loader(
        LoadedSource {
            id: "main.veac".to_owned(),
            source: source.to_owned(),
        },
        &MemoryLoader::default(),
    )
}

/// Validates and canonically formats a loaded source with its import loader.
pub fn format_source_with_loader(
    source: LoadedSource,
    loader: &dyn SourceLoader,
) -> Result<String, Diagnostics> {
    let kind = validation::validate(&source, loader)?;
    let tokens = lexer::lex(&source.id, &source.source).map_err(Diagnostics)?;
    let formatted = layout::format(&source.source, &tokens);
    let formatted_tokens = lexer::lex(&source.id, &formatted).map_err(Diagnostics)?;
    if !same_tokens(&tokens, &formatted_tokens) {
        return Err(Diagnostics::one(Diagnostic::new(
            "PROGRAM_FORMAT_TOKEN_CHANGE",
            &source.id,
            "formatter changed a non-trivia token",
            Span::default(),
        )));
    }
    if !preservation::same_comments(&source.source, &tokens, &formatted, &formatted_tokens) {
        return Err(Diagnostics::one(Diagnostic::new(
            "PROGRAM_FORMAT_COMMENT_CHANGE",
            &source.id,
            "formatter changed a comment or its token-relative ownership",
            Span::default(),
        )));
    }
    validation::validate_as(
        LoadedSource {
            id: source.id,
            source: formatted.clone(),
        },
        loader,
        kind,
    )?;
    Ok(formatted)
}

fn same_tokens(left: &[super::token::Token], right: &[super::token::Token]) -> bool {
    left.iter()
        .map(|token| &token.kind)
        .eq(right.iter().map(|token| &token.kind))
}

fn load_error(path: &Path, message: String) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_FORMAT_LOAD",
        path.display().to_string(),
        message,
        Span::default(),
    )
}

#[cfg(test)]
#[path = "formatter/tests.rs"]
mod tests;
