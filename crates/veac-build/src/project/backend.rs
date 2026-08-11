use std::{fmt, path::PathBuf};

use veac_artifact::{ArtifactEvidenceOutcome, VerifiedArtifact};
use veac_project::OutputId;

use crate::{CancellationToken, NodeId, PortName, ProjectAction};

mod identity;

pub use identity::*;

pub struct ProjectExecutionRequest<'a> {
    pub node_id: &'a NodeId,
    pub action: &'a ProjectAction,
    pub inputs: &'a [ProjectArtifactInput],
    pub workspace: &'a std::path::Path,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectArtifactInput {
    pub role: PortName,
    pub producer: NodeId,
    pub output: PortName,
    pub artifact: VerifiedArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProducedProjectOutput {
    pub output: OutputId,
    pub relative_path: PathBuf,
    pub semantics: ProjectOutputSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectOutputSemantics {
    Opaque,
    Evidence {
        suite_sha256: String,
        report_sha256: String,
        outcome: ArtifactEvidenceOutcome,
    },
}

pub trait ProjectBackend: Send + Sync {
    /// Return the closed identity contract for the selected project action backend.
    fn implementation_identity(
        &self,
        action: crate::ProjectActionKind,
    ) -> Result<ProjectBackendIdentity, ProjectBackendError>;

    fn execute(
        &self,
        request: ProjectExecutionRequest<'_>,
        cancellation: &CancellationToken,
    ) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectBackendErrorKind {
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectBackendError {
    kind: ProjectBackendErrorKind,
    message: String,
}

impl ProjectBackendError {
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            kind: ProjectBackendErrorKind::Failed,
            message: message.into(),
        }
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self {
            kind: ProjectBackendErrorKind::Cancelled,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ProjectBackendErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ProjectBackendError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ProjectBackendError {}
