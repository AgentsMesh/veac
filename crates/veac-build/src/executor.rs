use std::fmt::{Display, Formatter};

use veac_artifact::ContentDigest;

use crate::{ArtifactOutputs, BuildAction, CancellationToken, NodeCacheKey, NodeId, PortName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedInput {
    pub role: PortName,
    pub producer: NodeId,
    pub output: PortName,
    pub digest: ContentDigest,
}

#[derive(Debug)]
pub struct ExecuteRequest<'a, A> {
    pub node_id: &'a NodeId,
    pub action: &'a A,
    pub inputs: &'a [ResolvedInput],
    pub cache_key: &'a NodeCacheKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionErrorKind {
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionError {
    kind: ExecutionErrorKind,
    message: String,
}

impl ExecutionError {
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            kind: ExecutionErrorKind::Failed,
            message: message.into(),
        }
    }

    pub fn cancelled(message: impl Into<String>) -> Self {
        Self {
            kind: ExecutionErrorKind::Cancelled,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ExecutionErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ExecutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ExecutionError {}

pub trait NodeExecutor<A: BuildAction>: Send + Sync {
    /// Exact host implementation identity, captured before any computation-cache lookup.
    fn implementation_identity(&self, action: &A) -> crate::BuildResult<ContentDigest>;

    fn execute(
        &self,
        request: ExecuteRequest<'_, A>,
        cancellation: &CancellationToken,
    ) -> Result<ArtifactOutputs, ExecutionError>;
}
