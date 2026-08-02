use std::{fmt, io};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactErrorKind {
    InvalidContract,
    Serialization,
    Io,
    CorruptCache,
    MissingBinding,
    IdentityMismatch,
    ResourceLimit,
    UnsupportedIdentity,
    UnsafePath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactCommitState {
    NotCommitted,
    Committed,
}

#[derive(Debug)]
pub struct ArtifactError {
    pub kind: ArtifactErrorKind,
    pub commit_state: ArtifactCommitState,
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ArtifactError {
    pub(crate) fn new(kind: ArtifactErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            commit_state: ArtifactCommitState::NotCommitted,
            message: message.into(),
            source: None,
        }
    }

    pub(crate) fn with_source(
        kind: ArtifactErrorKind,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            commit_state: ArtifactCommitState::NotCommitted,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub(crate) fn committed(mut self) -> Self {
        self.commit_state = ArtifactCommitState::Committed;
        self
    }

    pub(crate) fn with_cleanup_failure(self, cleanup: ArtifactError, context: &str) -> Self {
        let commit_state = self.commit_state;
        let mut combined = Self::with_source(
            ArtifactErrorKind::UnsafePath,
            format!("{context}: {}", self.message),
            cleanup,
        );
        combined.commit_state = commit_state;
        combined
    }
}

impl fmt::Display for ArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)?;
        if self.commit_state == ArtifactCommitState::Committed {
            formatter.write_str(
                "; publication crossed the commit point; destination state is uncertain; do not retry blindly",
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for ArtifactError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

impl From<io::Error> for ArtifactError {
    fn from(value: io::Error) -> Self {
        Self::with_source(
            ArtifactErrorKind::Io,
            "artifact filesystem operation failed",
            value,
        )
    }
}

pub type ArtifactResult<T> = Result<T, ArtifactError>;
