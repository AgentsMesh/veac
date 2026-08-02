use veac_plan::PlanInputId;

use super::super::error::{diagnostic, CodegenErrorKind};
use super::super::CodegenErrors;

pub(super) fn missing(id: &PlanInputId, role: Option<&str>) -> CodegenErrors {
    let message = role.map_or_else(
        || "resolved input has no machine-local binding".to_owned(),
        |role| format!("resolved input has no {role} binding"),
    );
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::MissingInputBinding,
        "INPUT_BINDING_MISSING",
        Some(id.to_string()),
        message,
    ))
}

pub(super) fn invalid(message: &str) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::InvalidResourceBinding,
        "RESOURCE_BINDING_INVALID",
        None,
        message,
    ))
}
