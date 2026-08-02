use std::fmt;

use veac_plan::ResolvedRenderPlan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenErrorKind {
    MissingInputBinding,
    MissingOutputBinding,
    MissingSequence,
    MissingInput,
    UnsupportedSource,
    UnsupportedEffect,
    UnsupportedAudioProcessing,
    UnsupportedColorProcessing,
    UnsupportedCaptionFeature,
    InvalidResourceBinding,
    InvalidPlan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenDiagnostic {
    pub kind: CodegenErrorKind,
    pub code: &'static str,
    pub object_id: Option<String>,
    pub location: String,
    pub message: String,
    pub suggested_repair: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenErrors {
    diagnostics: Vec<CodegenDiagnostic>,
}

impl CodegenErrors {
    pub fn one(diagnostic: CodegenDiagnostic) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    pub fn diagnostics(&self) -> &[CodegenDiagnostic] {
        &self.diagnostics
    }

    pub(super) fn new(diagnostics: Vec<CodegenDiagnostic>) -> Option<Self> {
        (!diagnostics.is_empty()).then_some(Self { diagnostics })
    }
}

impl fmt::Display for CodegenErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.diagnostics.first() {
            Some(first) => write!(
                formatter,
                "{} codegen error(s); first is {}: {}",
                self.diagnostics.len(),
                first.code,
                first.message
            ),
            None => formatter.write_str("codegen failed without a diagnostic"),
        }
    }
}

impl std::error::Error for CodegenErrors {}

pub(super) fn diagnostic(
    kind: CodegenErrorKind,
    code: &'static str,
    object_id: Option<String>,
    message: impl Into<String>,
) -> CodegenDiagnostic {
    let location = object_id.as_ref().map_or_else(
        || "render-plan:root".to_owned(),
        |id| format!("render-plan:object:{id}"),
    );
    CodegenDiagnostic {
        kind,
        code,
        object_id,
        location,
        message: message.into(),
        suggested_repair: Some(repair(kind).to_owned()),
    }
}

fn repair(kind: CodegenErrorKind) -> &'static str {
    match kind {
        CodegenErrorKind::MissingInputBinding => "provide a verified machine-local input binding",
        CodegenErrorKind::MissingOutputBinding => "bind every authored deliverable output",
        CodegenErrorKind::MissingSequence | CodegenErrorKind::MissingInput => {
            "regenerate the render plan from validated canonical IR"
        }
        CodegenErrorKind::UnsupportedSource
        | CodegenErrorKind::UnsupportedEffect
        | CodegenErrorKind::UnsupportedAudioProcessing
        | CodegenErrorKind::UnsupportedColorProcessing
        | CodegenErrorKind::UnsupportedCaptionFeature => {
            "remove the unsupported mechanism or select a capable backend"
        }
        CodegenErrorKind::InvalidResourceBinding => {
            "regenerate and verify the machine-local execution bindings"
        }
        CodegenErrorKind::InvalidPlan => "regenerate the render plan from canonical IR",
    }
}

pub(super) fn missing_entry(plan: &ResolvedRenderPlan) -> CodegenDiagnostic {
    diagnostic(
        CodegenErrorKind::MissingSequence,
        "ENTRY_SEQUENCE_MISSING",
        Some(plan.entry_sequence_id.to_string()),
        "resolved plan entry sequence is absent",
    )
}

#[cfg(test)]
mod tests;
