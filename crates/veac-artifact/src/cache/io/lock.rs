use std::fs::File;
use std::thread;
use std::time::Duration;

use rustix::fs::{flock, FlockOperation};

use super::common::io_error;
use crate::ArtifactResult;

pub(super) struct DirectoryLock {
    file: File,
}

impl DirectoryLock {
    pub(super) fn shared_while(
        directory: &File,
        guard: impl FnMut() -> bool,
    ) -> ArtifactResult<Self> {
        Self::acquire(directory, FlockOperation::NonBlockingLockShared, guard)
    }

    pub(super) fn exclusive_while(
        directory: &File,
        guard: impl FnMut() -> bool,
    ) -> ArtifactResult<Self> {
        Self::acquire(directory, FlockOperation::NonBlockingLockExclusive, guard)
    }

    fn acquire(
        directory: &File,
        operation: FlockOperation,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<Self> {
        let file = directory.try_clone()?;
        loop {
            crate::cache::guard::check(&mut guard)?;
            match flock(&file, operation) {
                Ok(()) => {
                    let lock = Self { file };
                    crate::cache::guard::check(&mut guard)?;
                    return Ok(lock);
                }
                Err(error) if error == rustix::io::Errno::WOULDBLOCK => {
                    thread::sleep(Duration::from_millis(1));
                }
                Err(rustix::io::Errno::INTR) => {}
                Err(error) => return Err(io_error("lock cache directory", error)),
            }
        }
    }
}

impl Drop for DirectoryLock {
    fn drop(&mut self) {
        let _ = flock(&self.file, FlockOperation::Unlock);
    }
}
