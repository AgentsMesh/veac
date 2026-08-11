use std::{
    fs::{File, OpenOptions},
    path::PathBuf,
    thread,
    time::Duration,
};

use fs2::FileExt;
use veac_artifact::ArtifactStore;

use super::{BuildCache, CacheReservation, NodeCacheKey};
use crate::{ArtifactOutputs, BuildError, BuildResult, CancellationToken};

const RECORD_VERSION: u32 = 1;

mod record;

use record::ComputationRecord;

pub struct DiskBuildCache {
    store: ArtifactStore,
    lease_root: PathBuf,
    poll_interval: Duration,
}

impl DiskBuildCache {
    pub fn new(store: ArtifactStore, lease_root: impl Into<PathBuf>) -> BuildResult<Self> {
        let lease_root = lease_root.into();
        std::fs::create_dir_all(&lease_root).map_err(io_error)?;
        let metadata = std::fs::symlink_metadata(&lease_root).map_err(io_error)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(BuildError::cache(
                "computation lease root must be a non-symlink directory",
            ));
        }
        let lease_root = std::fs::canonicalize(lease_root).map_err(io_error)?;
        Ok(Self {
            store,
            lease_root,
            poll_interval: Duration::from_millis(10),
        })
    }

    pub fn artifact_store(&self) -> &ArtifactStore {
        &self.store
    }

    fn lease_file(&self, key: &NodeCacheKey) -> BuildResult<File> {
        let value = &key.digest().value;
        let parent = self.lease_root.join(&value[..2]);
        std::fs::create_dir_all(&parent).map_err(io_error)?;
        OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(parent.join(format!("{}.lease", &value[2..])))
            .map_err(io_error)
    }
}

impl BuildCache for DiskBuildCache {
    fn get(&self, key: &NodeCacheKey) -> BuildResult<Option<ArtifactOutputs>> {
        let Some(cached) = self.store.get(key.digest()).map_err(artifact_error)? else {
            return Ok(None);
        };
        if cached.descriptor != *key.descriptor() {
            return Err(BuildError::cache(
                "computation descriptor does not match its key",
            ));
        }
        let record: ComputationRecord = serde_json::from_slice(&cached.payload)
            .map_err(|error| BuildError::cache(format!("invalid computation record: {error}")))?;
        record.open(key, &self.store).map(Some)
    }

    fn put(&self, key: &NodeCacheKey, outputs: &ArtifactOutputs) -> BuildResult<()> {
        let record = ComputationRecord::capture(key, outputs, &self.store)?;
        let bytes = serde_json_canonicalizer::to_vec(&record).map_err(|error| {
            BuildError::cache(format!("cannot encode computation record: {error}"))
        })?;
        let stored = self
            .store
            .put(key.descriptor(), &bytes)
            .map_err(artifact_error)?;
        if stored.key != *key.digest() {
            return Err(BuildError::cache(
                "stored computation record has the wrong key",
            ));
        }
        Ok(())
    }

    fn reserve(
        &self,
        key: &NodeCacheKey,
        cancellation: &CancellationToken,
    ) -> BuildResult<CacheReservation> {
        let file = self.lease_file(key)?;
        loop {
            if cancellation.is_cancelled() {
                return Err(BuildError::cancelled(
                    "cancelled while waiting for computation lease",
                ));
            }
            match file.try_lock_exclusive() {
                Ok(()) => {
                    if let Some(outputs) = self.get(key)? {
                        FileExt::unlock(&file).map_err(io_error)?;
                        return Ok(CacheReservation::Hit(outputs));
                    }
                    return Ok(CacheReservation::Owner(Box::new(DiskLease(file))));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(self.poll_interval);
                }
                Err(error) => return Err(io_error(error)),
            }
        }
    }
}

struct DiskLease(File);

impl Drop for DiskLease {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

fn artifact_error(error: veac_artifact::ArtifactError) -> BuildError {
    BuildError::cache(format!("artifact cache failure: {error}"))
}

fn io_error(error: std::io::Error) -> BuildError {
    BuildError::cache(format!("computation lease I/O failed: {error}"))
}

#[cfg(test)]
#[path = "disk/tests.rs"]
mod tests;
