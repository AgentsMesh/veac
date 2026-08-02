use veac_plan::ResolvedClip;

use crate::emitter::error::{diagnostic, CodegenErrorKind};
use crate::emitter::CodegenErrors;

#[derive(Debug)]
pub(super) struct TextError {
    pub code: &'static str,
    pub message: String,
}

impl TextError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new("TEXT_PLAN_INVALID", message)
    }

    pub fn binding(message: impl Into<String>) -> Self {
        Self::new("TEXT_FONT_BINDING_INVALID", message)
    }
}

pub(super) fn codegen(clip: &ResolvedClip, error: TextError) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        kind(error.code),
        error.code,
        Some(clip.id.to_string()),
        error.message,
    ))
}

fn kind(code: &str) -> CodegenErrorKind {
    if code.starts_with("TEXT_FONT_") {
        CodegenErrorKind::InvalidResourceBinding
    } else {
        CodegenErrorKind::InvalidPlan
    }
}

#[cfg(test)]
mod tests;
