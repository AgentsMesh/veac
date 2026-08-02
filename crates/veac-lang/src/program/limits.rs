use crate::authoring::Span;

use super::diagnostic::Diagnostic;

pub(crate) const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
const MAX_SOURCE_GRAPH_BYTES: usize = 64 * 1024 * 1024;
const MAX_SOURCE_MODULES: usize = 1024;
pub(crate) const MAX_SOURCE_TOKENS: usize = 1_000_000;

#[derive(Default)]
pub(super) struct SourceBudget {
    bytes: usize,
    modules: usize,
}

impl SourceBudget {
    pub(super) fn add(&mut self, path: &str, source: &str, span: Span) -> Result<(), Diagnostic> {
        check_source_size(path, source, span)?;
        if self.modules >= MAX_SOURCE_MODULES {
            return Err(limit(
                "PROGRAM_MODULE_LIMIT",
                path,
                "source graph exceeds 1024 modules",
                span,
            ));
        }
        let bytes = self.bytes.checked_add(source.len()).ok_or_else(|| {
            limit(
                "PROGRAM_SOURCE_GRAPH_LIMIT",
                path,
                "source graph byte count overflowed",
                span,
            )
        })?;
        if bytes > MAX_SOURCE_GRAPH_BYTES {
            return Err(limit(
                "PROGRAM_SOURCE_GRAPH_LIMIT",
                path,
                "source graph exceeds 64 MiB",
                span,
            ));
        }
        self.bytes = bytes;
        self.modules += 1;
        Ok(())
    }
}

pub(super) fn check_source_size(path: &str, source: &str, span: Span) -> Result<(), Diagnostic> {
    if source.len() > MAX_SOURCE_BYTES {
        Err(limit(
            "PROGRAM_SOURCE_LIMIT",
            path,
            "source module exceeds 16 MiB",
            span,
        ))
    } else {
        Ok(())
    }
}

fn limit(code: &'static str, path: &str, message: &str, span: Span) -> Diagnostic {
    Diagnostic::new(code, path, message, span)
}

#[cfg(test)]
#[path = "limits/tests.rs"]
mod tests;
