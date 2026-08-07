use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionErrorKind {
    CanonicalValidation,
    RenderConfigNotFound,
    RemoteMaterialUnresolved,
    MaterialIdentityMissing,
    MaterialProbeMissing,
    MaterialProbeIdentityMismatch,
    RequiredStreamMissing,
    StreamSelectionMismatch,
    FontFamilyUnresolved,
    FontMaterialUnresolved,
    SourceDurationUnavailable,
    TimelineDurationUnavailable,
    NestedSequenceComponentUnavailable,
    SourceRangeOutOfBounds,
    TimeArithmetic,
    RenderPlanContract,
    InternalInvariant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolutionDiagnostic {
    pub kind: ResolutionErrorKind,
    pub code: String,
    pub object_id: Option<String>,
    pub pointer: String,
    pub message: String,
    pub suggested_repair: Option<String>,
}

impl ResolutionDiagnostic {
    pub(crate) fn new(
        kind: ResolutionErrorKind,
        code: &str,
        object_id: Option<String>,
        pointer: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code: code.to_owned(),
            object_id,
            pointer: pointer.into(),
            message: message.into(),
            suggested_repair: Some(
                "correct the canonical IR or provider binding at the indicated pointer".to_owned(),
            ),
        }
    }

    pub(crate) fn repair(mut self, repair: &str) -> Self {
        self.suggested_repair = Some(repair.to_owned());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionErrors {
    diagnostics: Vec<ResolutionDiagnostic>,
}

impl ResolutionErrors {
    pub fn new(diagnostics: Vec<ResolutionDiagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[ResolutionDiagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<ResolutionDiagnostic> {
        self.diagnostics
    }
}

impl fmt::Display for ResolutionErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.diagnostics.first() {
            Some(first) => write!(
                formatter,
                "{} resolution error(s); first is {} at {}: {}",
                self.diagnostics.len(),
                first.code,
                first.pointer,
                first.message
            ),
            None => formatter.write_str("resolution failed without a diagnostic"),
        }
    }
}

impl std::error::Error for ResolutionErrors {}
