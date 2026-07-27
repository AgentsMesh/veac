use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use serde::{Deserialize, Serialize};
use veac_artifact::{ArtifactRecord, ArtifactStore};
use veac_provider::{ProviderManifest, ProviderRequestEnvelope, ProviderResponseEnvelope};

use crate::tool::PinnedExecutable;

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

mod identity;
mod limits;
mod process;
mod protocol;
mod runner;
mod staging;
#[cfg(test)]
#[path = "provider/tests.rs"]
mod tests;

pub use identity::ProviderExecutionIdentity;
pub use limits::ProviderResourceLimits;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderExecution {
    pub manifest: ProviderManifest,
    pub identity: ProviderExecutionIdentity,
    pub response: ProviderResponseEnvelope,
    pub artifacts: Vec<ArtifactRecord>,
}

#[derive(Debug, Clone)]
pub struct ProviderRunner {
    program: PathBuf,
    arguments: Vec<OsString>,
    pinned: Arc<OnceLock<PinnedExecutable>>,
    limits: ProviderResourceLimits,
}

impl ProviderRunner {
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            arguments: Vec::new(),
            pinned: Arc::new(OnceLock::new()),
            limits: ProviderResourceLimits::default(),
        }
    }

    pub fn with_arguments<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.arguments = arguments.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_limits(mut self, limits: ProviderResourceLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn run(
        &self,
        store: &ArtifactStore,
        request: &ProviderRequestEnvelope,
    ) -> WorkflowResult<ProviderExecution> {
        runner::run(self, store, request)
    }
}
