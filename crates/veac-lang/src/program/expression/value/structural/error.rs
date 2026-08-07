use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueConstructionError {
    code: &'static str,
    message: String,
}

impl ValueConstructionError {
    pub fn code(&self) -> &'static str {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub(in crate::program::expression::value) fn new(
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub(super) fn value_type(error: crate::program::expression::ValueTypeError) -> Self {
        Self::new(error.code(), error.message())
    }
}

impl fmt::Display for ValueConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ValueConstructionError {}
