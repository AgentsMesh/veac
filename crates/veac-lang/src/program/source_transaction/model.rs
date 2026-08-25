use std::fmt;

use std::collections::BTreeMap;

use crate::source_edit::{SourceEditError, SourceRevision};

use super::super::{BuiltProgram, Diagnostics, ExecutableBuild};

#[derive(Debug)]
pub enum SourceTransactionError {
    Program(Diagnostics),
    Contract(SourceEditError),
    ReadOnlySource { module: String },
    TargetNotFound { operation: usize },
    ChangedModuleUnreachable { module: String },
    RootChanged { expected: String, actual: String },
}

impl fmt::Display for SourceTransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Program(error) => write!(formatter, "{error}"),
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::ReadOnlySource { module } => {
                write!(
                    formatter,
                    "source module {module:?} is a read-only dependency"
                )
            }
            Self::TargetNotFound { operation } => write!(
                formatter,
                "source-edit operation {operation} has no authored source site"
            ),
            Self::ChangedModuleUnreachable { module } => write!(
                formatter,
                "changed source module {module:?} is unreachable in the candidate graph"
            ),
            Self::RootChanged { expected, actual } => write!(
                formatter,
                "source root changed from canonical module {expected:?} to {actual:?}"
            ),
        }
    }
}

impl std::error::Error for SourceTransactionError {}

#[derive(Debug)]
pub struct ExecutableSourceEditPreview {
    changes: Vec<SourceModuleChange>,
    pub previous_revision: SourceRevision,
    pub new_revision: SourceRevision,
    pub built: BuiltProgram,
    pub(super) previous_modules: Vec<String>,
    pub(super) previous_sources: BTreeMap<String, String>,
    pub(super) candidate_sources: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct ExecutableSourceEditCandidate {
    pub(super) changes: Vec<SourceModuleChange>,
    pub(super) prepared: ExecutableBuild,
    pub(super) previous_revision: SourceRevision,
    pub(super) previous_modules: Vec<String>,
    pub(super) previous_sources: BTreeMap<String, String>,
}

impl ExecutableSourceEditCandidate {
    pub(super) fn new(
        changes: Vec<SourceModuleChange>,
        prepared: ExecutableBuild,
        previous_revision: SourceRevision,
        previous_modules: Vec<String>,
        previous_sources: BTreeMap<String, String>,
    ) -> Self {
        Self {
            changes,
            prepared,
            previous_revision,
            previous_modules,
            previous_sources,
        }
    }

    pub fn program(&self) -> &ExecutableBuild {
        &self.prepared
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModuleChange {
    module: String,
    previous_source: String,
    source: String,
}

impl SourceModuleChange {
    pub(super) fn new(module: String, previous_source: String, source: String) -> Self {
        Self {
            module,
            previous_source,
            source,
        }
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    pub fn previous_source(&self) -> &str {
        &self.previous_source
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

impl ExecutableSourceEditPreview {
    pub(super) fn new(
        changes: Vec<SourceModuleChange>,
        previous_revision: SourceRevision,
        new_revision: SourceRevision,
        built: BuiltProgram,
        previous_modules: Vec<String>,
        previous_sources: BTreeMap<String, String>,
        candidate_sources: BTreeMap<String, String>,
    ) -> Self {
        Self {
            changes,
            previous_revision,
            new_revision,
            built,
            previous_modules,
            previous_sources,
            candidate_sources,
        }
    }

    pub fn changes(&self) -> &[SourceModuleChange] {
        &self.changes
    }

    pub fn previous_source(&self) -> Option<&str> {
        self.only_change().map(SourceModuleChange::previous_source)
    }

    pub fn source(&self) -> Option<&str> {
        self.only_change().map(SourceModuleChange::source)
    }

    pub fn changed_modules(&self) -> Vec<&str> {
        self.changes
            .iter()
            .map(SourceModuleChange::module)
            .collect()
    }

    pub fn previous_modules(&self) -> &[String] {
        &self.previous_modules
    }

    pub fn previous_sources(&self) -> &BTreeMap<String, String> {
        &self.previous_sources
    }

    pub fn candidate_sources(&self) -> &BTreeMap<String, String> {
        &self.candidate_sources
    }

    fn only_change(&self) -> Option<&SourceModuleChange> {
        let [change] = self.changes.as_slice() else {
            return None;
        };
        Some(change)
    }
}
