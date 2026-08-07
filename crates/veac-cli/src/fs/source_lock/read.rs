use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::FlockOperation;

use crate::error::CliResult;

use super::handle;

#[derive(Debug)]
pub(crate) struct SourceGraphReadLock {
    directory: OwnedFd,
    lock: OwnedFd,
}

impl SourceGraphReadLock {
    pub(crate) fn acquire(root: &Path) -> CliResult<Self> {
        let (directory, lock) =
            handle::acquire(root, FlockOperation::NonBlockingLockShared, "edited")?;
        Ok(Self { directory, lock })
    }

    pub(crate) fn revalidate(&self, root: &Path) -> CliResult {
        handle::revalidate(root, &self.directory, &self.lock)
    }
}

impl Drop for SourceGraphReadLock {
    fn drop(&mut self) {
        handle::unlock(&self.lock);
    }
}
