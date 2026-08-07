use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::FlockOperation;

use crate::error::CliResult;

macro_rules! cli_try {
    ($result:expr, |$error:ident| $mapped:expr) => {
        match $result {
            Ok(value) => value,
            Err($error) => return Err($mapped),
        }
    };
}

mod commit;
mod handle;
mod path;
mod read;
mod stage;

pub(crate) use commit::{SourceModuleGuard, SourceModuleReplacement};
pub(crate) use read::SourceGraphReadLock;

pub(crate) const SOURCE_LOCK_NAME: &str = ".veac-source.lock";

#[derive(Debug)]
pub(crate) struct SourceGraphLock {
    pub(super) directory: OwnedFd,
    lock: OwnedFd,
}

impl SourceGraphLock {
    pub(crate) fn acquire(root: &Path) -> CliResult<Self> {
        let (directory, lock) = handle::acquire(
            root,
            FlockOperation::NonBlockingLockExclusive,
            "read or edited",
        )?;
        Ok(Self { directory, lock })
    }

    pub(crate) fn revalidate(&self, root: &Path) -> CliResult {
        handle::revalidate(root, &self.directory, &self.lock)
    }
}

impl Drop for SourceGraphLock {
    fn drop(&mut self) {
        handle::unlock(&self.lock);
    }
}

#[cfg(test)]
#[path = "source_lock/tests.rs"]
mod tests;
