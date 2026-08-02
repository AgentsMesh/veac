use std::path::PathBuf;

use tempfile::TempDir;
use veac_artifact::DeliveryPackageInventory;

use super::directory::Directory;
use super::StaleFamily;

#[derive(Debug, Clone)]
pub(in crate::executor) struct StagedFile {
    pub source: PathBuf,
    pub target: PathBuf,
    pub allow_empty: bool,
}

#[derive(Debug, Clone)]
pub(in crate::executor) struct StagedPackage {
    pub source: PathBuf,
    pub target: PathBuf,
    pub entrypoint: PathBuf,
    pub inventory: DeliveryPackageInventory,
}

#[derive(Debug, Clone)]
pub(in crate::executor) enum StagedOutput {
    File(StagedFile),
    Package(StagedPackage),
}

impl From<StagedFile> for StagedOutput {
    fn from(value: StagedFile) -> Self {
        Self::File(value)
    }
}

impl StagedOutput {
    pub fn source(&self) -> &PathBuf {
        match self {
            Self::File(value) => &value.source,
            Self::Package(value) => &value.source,
        }
    }

    pub fn target(&self) -> &PathBuf {
        match self {
            Self::File(value) => &value.target,
            Self::Package(value) => &value.target,
        }
    }

    pub fn rebase_source(&mut self, source: PathBuf) {
        match self {
            Self::File(value) => value.source = source,
            Self::Package(value) => value.source = source,
        }
    }
}

pub(in crate::executor) struct StagedTask {
    pub(super) directory: TempDir,
    pub(super) descriptor: Directory,
    pub(super) outputs: Vec<StagedOutput>,
    pub(super) stale: Vec<StaleFamily>,
}

impl StagedTask {
    pub fn outputs(&self) -> &[StagedOutput] {
        &self.outputs
    }

    pub fn targets(&self) -> Vec<PathBuf> {
        self.outputs
            .iter()
            .map(|value| value.target().clone())
            .collect()
    }

    pub fn sources(&self) -> Vec<PathBuf> {
        self.outputs
            .iter()
            .map(|value| value.source().clone())
            .collect()
    }
}
