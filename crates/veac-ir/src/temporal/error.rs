use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalDiagnostic {
    pub code: String,
    pub pointer: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalValidationErrors {
    diagnostics: Vec<TemporalDiagnostic>,
}

impl TemporalValidationErrors {
    pub(crate) fn new(diagnostics: Vec<TemporalDiagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[TemporalDiagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<TemporalDiagnostic> {
        self.diagnostics
    }
}

impl fmt::Display for TemporalValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let first = &self.diagnostics[0];
        write!(
            formatter,
            "{} temporal diagnostic(s); first is {} at {}: {}",
            self.diagnostics.len(),
            first.code,
            first.pointer,
            first.message
        )
    }
}

impl std::error::Error for TemporalValidationErrors {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalEvaluationError {
    code: String,
    pointer: String,
    message: String,
}

impl TemporalEvaluationError {
    pub(crate) fn new(
        code: impl Into<String>,
        pointer: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            pointer: pointer.into(),
            message: message.into(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn pointer(&self) -> &str {
        &self.pointer
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for TemporalEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.pointer, self.message
        )
    }
}

impl std::error::Error for TemporalEvaluationError {}
