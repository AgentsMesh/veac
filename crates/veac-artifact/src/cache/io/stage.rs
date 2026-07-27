use std::io::Write;

use super::common::corrupt;
use super::directory::BoundDirectory;
use super::entry::BoundFile;
use super::lock::DirectoryLock;
use super::state::{self, DESCRIPTOR, PAYLOAD, RECORD};
use crate::{
    ArtifactError, ArtifactResult, ContentDigest, MAX_ARTIFACT_METADATA_BYTES,
    MAX_ARTIFACT_PAYLOAD_BYTES,
};

mod create;
mod guarded;

pub(super) enum StageOpen {
    Ready(Box<CacheStage>),
    Conflict,
}

pub(super) struct CacheStage {
    directory: BoundDirectory,
    descriptor: BoundFile,
    payload: BoundFile,
    record: BoundFile,
    sealed: bool,
    _prefix_lock: DirectoryLock,
}

pub(super) struct ExpectedContent<'a> {
    pub descriptor: &'a [u8],
    pub record: &'a [u8],
    pub payload_digest: &'a ContentDigest,
    pub payload_size: u64,
}

impl CacheStage {
    pub(super) fn begin(
        parent: super::authority::BoundChain,
        target: std::ffi::OsString,
        guard: impl FnMut() -> bool,
    ) -> ArtifactResult<StageOpen> {
        create::begin(parent, target, guard)
    }

    pub(super) fn path(&self) -> &std::path::Path {
        self.directory.path()
    }

    pub(super) fn write_metadata(
        &mut self,
        descriptor: &[u8],
        record: &[u8],
    ) -> ArtifactResult<()> {
        self.descriptor.file_mut().write_all(descriptor)?;
        self.record.file_mut().write_all(record)?;
        Ok(())
    }

    pub(super) fn payload(&mut self) -> &mut std::fs::File {
        self.payload.file_mut()
    }

    fn require_data_entries(&self) -> ArtifactResult<()> {
        if self.directory.entries(4)? != state::data_entries() {
            return corrupt("unsealed cache directory contains unexpected entries");
        }
        Ok(())
    }

    fn cleanup_after(&mut self, primary: ArtifactError) -> ArtifactError {
        match self.cleanup() {
            Ok(()) => primary,
            Err(cleanup) => {
                primary.with_cleanup_failure(cleanup, "cache transaction cleanup was unsafe")
            }
        }
    }

    fn cleanup(&mut self) -> ArtifactResult<()> {
        if self.sealed {
            return Ok(());
        }
        self.directory.verify()?;
        self.require_data_entries()?;
        self.descriptor.unlink(self.directory.file())?;
        self.payload.unlink(self.directory.file())?;
        self.record.unlink(self.directory.file())?;
        self.directory.remove_empty()?;
        self.directory.sync_parent()?;
        self.sealed = true;
        Ok(())
    }
}

impl Drop for CacheStage {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
