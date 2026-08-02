use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionError {
    code: &'static str,
    message: String,
    span: Range<usize>,
}

impl ExpressionError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>, span: Range<usize>) -> Self {
        Self {
            code,
            message: message.into(),
            span,
        }
    }

    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn span(&self) -> Range<usize> {
        self.span.clone()
    }
}

impl fmt::Display for ExpressionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}..{}: {}",
            self.code, self.span.start, self.span.end, self.message
        )
    }
}

impl std::error::Error for ExpressionError {}
