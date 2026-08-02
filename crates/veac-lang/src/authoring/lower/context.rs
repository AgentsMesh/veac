use crate::authoring::{Diagnostic, Diagnostics, Span};
use std::collections::HashSet;

pub(super) struct Context {
    pub timescale: u32,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub audio_resources: HashSet<String>,
    pub audio_sequences: HashSet<String>,
    diagnostics: Vec<Diagnostic>,
}

impl Context {
    pub fn new(timescale: u32) -> Self {
        Self {
            timescale,
            canvas_width: 1.0,
            canvas_height: 1.0,
            audio_resources: HashSet::new(),
            audio_sequences: HashSet::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        crate::authoring::diagnostic_budget::push(
            &mut self.diagnostics,
            Diagnostic::new(code, message, span),
        );
    }

    pub fn unsupported(&mut self, mechanism: &str, span: Span) {
        self.error(
            "AUTHORING_LOWER_UNSUPPORTED",
            format!("{mechanism} is not representable in canonical IR"),
            span,
        );
    }

    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn diagnostics(self) -> Diagnostics {
        self.diagnostics.into()
    }
}
