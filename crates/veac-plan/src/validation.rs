mod graph;
mod header;
mod temporal;

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ResolvedRenderPlan;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanValidationDiagnostic {
    pub code: String,
    pub pointer: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanValidationErrors {
    diagnostics: Vec<PlanValidationDiagnostic>,
}

impl PlanValidationErrors {
    pub fn diagnostics(&self) -> &[PlanValidationDiagnostic] {
        &self.diagnostics
    }

    pub fn into_diagnostics(self) -> Vec<PlanValidationDiagnostic> {
        self.diagnostics
    }
}

impl fmt::Display for PlanValidationErrors {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(first) = self.diagnostics.first() else {
            return formatter.write_str("render plan validation failed without diagnostics");
        };
        write!(
            formatter,
            "{} render plan diagnostic(s); first is {} at {}: {}",
            self.diagnostics.len(),
            first.code,
            first.pointer,
            first.message
        )
    }
}

impl std::error::Error for PlanValidationErrors {}

#[derive(Default)]
pub(super) struct Validator {
    diagnostics: Vec<PlanValidationDiagnostic>,
}

impl Validator {
    pub(super) fn push(
        &mut self,
        code: &str,
        pointer: impl Into<String>,
        message: impl Into<String>,
    ) {
        self.diagnostics.push(PlanValidationDiagnostic {
            code: code.to_owned(),
            pointer: pointer.into(),
            message: message.into(),
        });
    }
}

pub fn validate_render_plan(plan: &ResolvedRenderPlan) -> Result<(), PlanValidationErrors> {
    let mut validator = Validator::default();
    validator.header(plan);
    validator.graph(plan);
    validator.temporal(plan);
    validator.diagnostics.sort_by(|left, right| {
        (&left.pointer, &left.code, &left.message).cmp(&(
            &right.pointer,
            &right.code,
            &right.message,
        ))
    });
    validator.diagnostics.dedup();
    if validator.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(PlanValidationErrors {
            diagnostics: validator.diagnostics,
        })
    }
}

pub(super) fn sha256_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
