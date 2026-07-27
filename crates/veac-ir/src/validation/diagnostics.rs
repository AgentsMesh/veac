use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::Validator;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    pub code: String,
    pub object_id: Option<String>,
    pub pointer: String,
    pub message: String,
    pub suggested_repair: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationErrors {
    pub(super) fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let diagnostic = &self.diagnostics[0];
        write!(
            formatter,
            "{} diagnostic(s); first is {} at {}: {}",
            self.diagnostics.len(),
            diagnostic.code,
            diagnostic.pointer,
            diagnostic.message
        )
    }
}

impl std::error::Error for ValidationErrors {}

impl Validator {
    pub(super) fn check_id(&mut self, valid: bool, id: &str, path: &str) {
        if !valid {
            self.push(
                "INVALID_ID",
                Some(id.to_owned()),
                path,
                "identifier has the wrong prefix or unsupported characters",
                None,
            );
        }
    }

    pub(super) fn duplicate(&mut self, code: &str, id: &str, path: &str) {
        self.push(
            code,
            Some(id.to_owned()),
            path,
            format!("identifier {id:?} is duplicated"),
            Some("assign a stable unique identifier"),
        );
    }

    pub(super) fn missing_ref(&mut self, code: &str, id: &str, path: &str) {
        self.push(
            code,
            Some(id.to_owned()),
            path,
            format!("referenced object {id:?} does not exist"),
            None,
        );
    }

    pub(super) fn value_error(&mut self, code: &str, path: &str, object_id: &str) {
        self.push(
            code,
            Some(object_id.to_owned()),
            path,
            "value is outside the canonical IR contract",
            None,
        );
    }

    pub(super) fn push(
        &mut self,
        code: impl Into<String>,
        object_id: Option<String>,
        pointer: impl Into<String>,
        message: impl Into<String>,
        suggested_repair: Option<&str>,
    ) {
        self.diagnostics.push(Diagnostic {
            code: code.into(),
            object_id,
            pointer: pointer.into(),
            message: message.into(),
            suggested_repair: Some(
                suggested_repair
                    .unwrap_or("correct the value at the indicated canonical JSON pointer")
                    .to_owned(),
            ),
        });
    }
}
