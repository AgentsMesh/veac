use crate::error::CliError;
use std::path::Path;

pub(crate) mod model;
pub use model::{CliDiagnostic, DiagnosticFormat, SourceSpan};

pub(crate) fn authoring(
    path: &Path,
    source: &str,
    errors: veac_lang::authoring::Diagnostics,
) -> CliError {
    let diagnostics = errors
        .as_slice()
        .iter()
        .map(|diagnostic| {
            let (line, column) = line_column(source, diagnostic.span.start);
            CliDiagnostic {
                code: diagnostic.code.to_owned(),
                object_id: None,
                source_span: Some(SourceSpan {
                    path: path.display().to_string(),
                    start: diagnostic.span.start,
                    end: diagnostic.span.end,
                    line,
                    column,
                }),
                pointer: None,
                location: None,
                message: diagnostic.message.clone(),
                suggested_repair: None,
            }
        })
        .collect();
    CliError::from_diagnostics(diagnostics)
}

pub(crate) fn program(path: &Path, errors: veac_lang::program::Diagnostics) -> CliError {
    let root = path.parent().unwrap_or_else(|| Path::new("."));
    let canonical_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let diagnostics = errors
        .as_slice()
        .iter()
        .map(|diagnostic| {
            let requested = canonical_root.join(&diagnostic.path);
            let source = std::fs::canonicalize(&requested)
                .ok()
                .filter(|path| path.starts_with(&canonical_root))
                .and_then(|path| {
                    crate::fs::read_utf8_bounded(&path, "diagnostic source", 16 * 1024 * 1024).ok()
                })
                .unwrap_or_default();
            let (line, column) = line_column(&source, diagnostic.span.start);
            CliDiagnostic {
                code: program_code(diagnostic.code).to_owned(),
                object_id: None,
                source_span: Some(SourceSpan {
                    path: root.join(&diagnostic.path).display().to_string(),
                    start: diagnostic.span.start,
                    end: diagnostic.span.end,
                    line,
                    column,
                }),
                pointer: None,
                location: None,
                message: diagnostic.message.clone(),
                suggested_repair: None,
            }
        })
        .collect();
    CliError::from_diagnostics(diagnostics)
}

fn program_code(code: &'static str) -> &'static str {
    match code {
        "PROGRAM_ENTRY_LOAD" => "READ_FAILED",
        other => other,
    }
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let prefix = source.get(..offset.min(source.len())).unwrap_or(source);
    let line = prefix.bytes().filter(|value| *value == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix.chars().count() + 1, |(_, tail)| {
            tail.chars().count() + 1
        });
    (line, column)
}

pub(crate) fn resolution(errors: veac_plan::ResolutionErrors) -> CliError {
    let diagnostics = errors
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| CliDiagnostic {
            code: diagnostic.code,
            object_id: diagnostic.object_id,
            source_span: None,
            pointer: Some(diagnostic.pointer),
            location: None,
            message: diagnostic.message,
            suggested_repair: diagnostic.suggested_repair,
        })
        .collect();
    CliError::from_diagnostics(diagnostics)
}

pub(crate) fn codegen(errors: veac_codegen::emitter::CodegenErrors) -> CliError {
    let diagnostics = errors
        .diagnostics()
        .iter()
        .map(|diagnostic| CliDiagnostic {
            code: diagnostic.code.to_owned(),
            object_id: diagnostic.object_id.clone(),
            source_span: None,
            pointer: None,
            location: Some(diagnostic.location.clone()),
            message: diagnostic.message.clone(),
            suggested_repair: diagnostic.suggested_repair.clone(),
        })
        .collect();
    CliError::from_diagnostics(diagnostics)
}

pub(crate) fn ir(diagnostics: &[veac_ir::Diagnostic]) -> CliError {
    let diagnostics = diagnostics
        .iter()
        .map(|diagnostic| CliDiagnostic {
            code: diagnostic.code.clone(),
            object_id: diagnostic.object_id.clone(),
            source_span: None,
            pointer: Some(diagnostic.pointer.clone()),
            location: None,
            message: diagnostic.message.clone(),
            suggested_repair: diagnostic.suggested_repair.clone(),
        })
        .collect();
    CliError::from_diagnostics(diagnostics)
}
