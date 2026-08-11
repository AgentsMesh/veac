pub mod asset;
pub mod executor;
pub mod observation;
pub mod progress;
pub mod workflow;

mod identity;
pub use identity::{artifact_backend_identity, runtime_backend_identity};

mod input_policy;
mod process_group;
mod tool;

use std::fmt;

/// Runtime error for FFmpeg execution and media probing.
#[derive(Debug)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    General,
    MissingBackendCapability,
    ResourceLimit,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeErrorKind::General,
            message: message.into(),
        }
    }

    pub(crate) fn missing_backend_capability(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeErrorKind::MissingBackendCapability,
            message: message.into(),
        }
    }

    pub fn resource_limit(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeErrorKind::ResourceLimit,
            message: message.into(),
        }
    }

    pub(crate) fn context(self, context: &str) -> Self {
        Self {
            kind: self.kind,
            message: format!("{context}: {}", self.message),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod runtime_error_tests;
