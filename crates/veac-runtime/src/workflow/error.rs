use std::{fmt, io};

#[cfg(test)]
#[path = "error/tests.rs"]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowErrorKind {
    InvalidContract,
    ResourceLimit,
    SourceIdentityMismatch,
    UnsupportedOperation,
    ToolFailure,
    ProtocolViolation,
    UnsafeStaging,
    Io,
    Artifact,
    Provider,
    Serialization,
}

#[derive(Debug)]
pub struct WorkflowError {
    pub kind: WorkflowErrorKind,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl WorkflowError {
    pub(super) fn new(kind: WorkflowErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            source: None,
        }
    }

    pub(super) fn with_source(
        kind: WorkflowErrorKind,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for WorkflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

impl From<io::Error> for WorkflowError {
    fn from(value: io::Error) -> Self {
        Self::with_source(
            WorkflowErrorKind::Io,
            "workflow filesystem operation failed",
            value,
        )
    }
}

impl From<veac_artifact::ArtifactError> for WorkflowError {
    fn from(value: veac_artifact::ArtifactError) -> Self {
        Self::with_source(
            WorkflowErrorKind::Artifact,
            "artifact store operation failed",
            value,
        )
    }
}

impl From<veac_provider::ProviderError> for WorkflowError {
    fn from(value: veac_provider::ProviderError) -> Self {
        Self::with_source(
            WorkflowErrorKind::Provider,
            "provider contract validation failed",
            value,
        )
    }
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;
