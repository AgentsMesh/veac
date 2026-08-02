use std::fmt;

use crate::authoring::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub path: String,
    pub message: String,
    pub span: Span,
}

impl Diagnostic {
    pub(crate) fn new(
        code: &'static str,
        path: impl Into<String>,
        message: impl Into<String>,
        span: Span,
    ) -> Self {
        Self {
            code,
            path: path.into(),
            message: message.into(),
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostics(pub(crate) Vec<Diagnostic>);

impl Diagnostics {
    pub fn as_slice(&self) -> &[Diagnostic] {
        &self.0
    }

    pub(crate) fn one(diagnostic: Diagnostic) -> Self {
        Self(vec![diagnostic])
    }
}

impl fmt::Display for Diagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in &self.0 {
            writeln!(
                formatter,
                "{}:{}..{}: {}",
                diagnostic.path, diagnostic.span.start, diagnostic.span.end, diagnostic.message
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for Diagnostics {}
