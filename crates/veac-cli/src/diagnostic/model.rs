use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiagnosticFormat {
    #[default]
    Human,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SourceSpan {
    pub path: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CliDiagnostic {
    pub code: String,
    pub object_id: Option<String>,
    pub source_span: Option<SourceSpan>,
    pub pointer: Option<String>,
    pub location: Option<String>,
    pub message: String,
    pub suggested_repair: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct DiagnosticEnvelope<'a> {
    pub diagnostics: &'a [CliDiagnostic],
}

impl CliDiagnostic {
    pub(crate) fn human(&self) -> String {
        let mut rendered = match self.primary_location() {
            Some(location) => format!("error[{}]: {location}: {}", self.code, self.message),
            None => format!("error[{}]: {}", self.code, self.message),
        };
        if let Some(id) = &self.object_id {
            rendered.push_str(&format!("\n  object: {id}"));
        }
        if self.source_span.is_some() {
            if let Some(pointer) = &self.pointer {
                rendered.push_str(&format!("\n  pointer: {pointer}"));
            }
        }
        if let Some(repair) = &self.suggested_repair {
            rendered.push_str(&format!("\n  help: {repair}"));
        }
        rendered
    }

    fn primary_location(&self) -> Option<String> {
        if let Some(span) = &self.source_span {
            return Some(format!("{}:{}:{}", span.path, span.line, span.column));
        }
        self.pointer.clone().or_else(|| self.location.clone())
    }
}
