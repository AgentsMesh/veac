use veac_ir::{CanonicalError, ValidationErrors};

use super::PlanResolver;
use crate::{ResolutionDiagnostic, ResolutionErrorKind, ResolutionErrors};

pub(super) fn canonical_errors(errors: ValidationErrors) -> ResolutionErrors {
    let diagnostics = errors
        .into_diagnostics()
        .into_iter()
        .map(|diagnostic| ResolutionDiagnostic {
            kind: canonical_kind(&diagnostic.code),
            code: diagnostic.code,
            object_id: diagnostic.object_id,
            pointer: diagnostic.pointer,
            message: diagnostic.message,
            suggested_repair: diagnostic.suggested_repair,
        })
        .collect();
    ResolutionErrors::new(diagnostics)
}

pub(super) fn canonical_kind(code: &str) -> ResolutionErrorKind {
    match code {
        "PROBE_IDENTITY" => ResolutionErrorKind::MaterialProbeIdentityMismatch,
        "PROBE_INTENT" | "PROBE_KIND" => ResolutionErrorKind::StreamSelectionMismatch,
        _ => ResolutionErrorKind::CanonicalValidation,
    }
}

pub(super) fn nested_component_missing(
    path: &str,
    sequence_id: &str,
    component: &str,
) -> ResolutionDiagnostic {
    let code = format!("NESTED_SEQUENCE_{}_UNAVAILABLE", component.to_uppercase());
    ResolutionDiagnostic::new(
        ResolutionErrorKind::NestedSequenceComponentUnavailable,
        &code,
        Some(sequence_id.to_owned()),
        path,
        format!("nested sequence {sequence_id} has no rendered {component} component"),
    )
}

pub(super) fn internal_hash_error(error: CanonicalError) -> ResolutionErrors {
    ResolutionErrors::new(vec![ResolutionDiagnostic::new(
        ResolutionErrorKind::InternalInvariant,
        "CANONICAL_HASH_FAILED",
        None,
        "/project",
        error.to_string(),
    )])
}

pub(super) fn normalize_diagnostics(diagnostics: &mut Vec<ResolutionDiagnostic>) {
    diagnostics.sort_by(|left, right| {
        (&left.pointer, &left.code, &left.object_id).cmp(&(
            &right.pointer,
            &right.code,
            &right.object_id,
        ))
    });
    diagnostics.dedup();
}

impl PlanResolver<'_> {
    pub(super) fn push(&mut self, diagnostic: ResolutionDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub(super) fn push_internal(&mut self, code: &str, id: String, message: String) {
        self.push(ResolutionDiagnostic::new(
            ResolutionErrorKind::InternalInvariant,
            code,
            Some(id),
            "/project",
            message,
        ));
    }
}
