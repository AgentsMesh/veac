use crate::diagnostic::{model::DiagnosticEnvelope, CliDiagnostic, DiagnosticFormat};
use std::fmt;

pub type CliResult<T = ()> = Result<T, CliError>;

#[derive(Debug)]
pub struct CliError {
    message: String,
    diagnostics: Vec<CliDiagnostic>,
    diagnostic_format: DiagnosticFormat,
    resource_limit: bool,
}

impl CliError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        let diagnostic = CliDiagnostic {
            code: code.into(),
            object_id: None,
            source_span: None,
            pointer: None,
            location: None,
            message: message.into(),
            suggested_repair: Some("correct the reported condition and retry".to_owned()),
        };
        Self::from_diagnostics(vec![diagnostic])
    }

    pub fn rendered(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            diagnostics: Vec::new(),
            diagnostic_format: DiagnosticFormat::Human,
            resource_limit: false,
        }
    }

    pub fn resource_limit(code: impl Into<String>, message: impl Into<String>) -> Self {
        let mut error = Self::new(code, message);
        error.diagnostics[0].suggested_repair =
            Some("reduce the requested resource usage and retry".to_owned());
        error.message = error.diagnostics[0].human();
        error.resource_limit = true;
        error
    }

    pub(crate) fn from_diagnostics(diagnostics: Vec<CliDiagnostic>) -> Self {
        let message = diagnostics
            .iter()
            .map(CliDiagnostic::human)
            .collect::<Vec<_>>()
            .join("\n");
        Self {
            message,
            diagnostics,
            diagnostic_format: DiagnosticFormat::Human,
            resource_limit: false,
        }
    }

    pub(crate) fn with_diagnostic_format(mut self, format: DiagnosticFormat) -> Self {
        self.diagnostic_format = format;
        self
    }

    pub fn diagnostics(&self) -> &[CliDiagnostic] {
        &self.diagnostics
    }

    pub fn diagnostics_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&DiagnosticEnvelope {
            diagnostics: &self.diagnostics,
        })
    }

    pub fn uses_json_diagnostics(&self) -> bool {
        self.diagnostic_format == DiagnosticFormat::Json && !self.diagnostics.is_empty()
    }

    pub fn is_resource_limit(&self) -> bool {
        self.resource_limit
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.diagnostic_format {
            DiagnosticFormat::Human => formatter.write_str(&self.message),
            DiagnosticFormat::Json if !self.diagnostics.is_empty() => {
                formatter.write_str(&self.diagnostics_json().map_err(|_| fmt::Error)?)
            }
            DiagnosticFormat::Json => formatter.write_str(&self.message),
        }
    }
}

impl std::error::Error for CliError {}
