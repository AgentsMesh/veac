use std::path::{Path, PathBuf};
use std::time::Instant;

use rustix::fs::{flock, open, openat, FileType, FlockOperation, Mode, OFlags};
use rustix::io::Errno;

use super::staging::directory::Directory;
use crate::RuntimeError;

const LOCK_NAME: &str = ".veac-render.lock";

#[derive(Clone, Copy, PartialEq, Eq)]
enum FailurePoint {
    InspectLockedDirectory,
    InspectCurrentDirectory,
    DuplicateDirectory,
    InspectLock,
    AcquireLock,
}

trait Operations {
    fn before(&self, point: FailurePoint) -> Result<(), Errno>;
}

struct LiveOperations;

impl Operations for LiveOperations {
    fn before(&self, _point: FailurePoint) -> Result<(), Errno> {
        Ok(())
    }
}

#[derive(Debug)]
pub(super) struct OutputLocks {
    directories: Vec<LockedDirectory>,
}

#[derive(Debug)]
struct LockedDirectory {
    path: PathBuf,
    directory: rustix::fd::OwnedFd,
    lock: rustix::fd::OwnedFd,
}

impl Drop for LockedDirectory {
    fn drop(&mut self) {
        let _ = flock(&self.lock, FlockOperation::Unlock);
    }
}

pub(super) fn acquire_until(
    parents: &[PathBuf],
    deadline: Instant,
) -> Result<OutputLocks, RuntimeError> {
    let mut directories = Vec::with_capacity(parents.len());
    for parent in parents {
        super::deadline::ensure_setup(deadline)?;
        directories.push(lock(parent, &LiveOperations)?);
        super::deadline::ensure_setup(deadline)?;
    }
    Ok(OutputLocks { directories })
}

impl OutputLocks {
    pub(super) fn directory(&self, parent: &Path) -> Result<Directory, RuntimeError> {
        let canonical = std::fs::canonicalize(parent).map_err(|error| {
            RuntimeError::new(format!("cannot revalidate output directory: {error}"))
        })?;
        self.directory_at_canonical_path(&canonical, &LiveOperations)
    }

    pub(super) fn directory_until(
        &self,
        parent: &Path,
        deadline: Instant,
    ) -> Result<Directory, RuntimeError> {
        super::deadline::ensure(deadline)?;
        let directory = self.directory(parent)?;
        super::deadline::ensure(deadline)?;
        Ok(directory)
    }

    pub(super) fn setup_directory_until(
        &self,
        parent: &Path,
        deadline: Instant,
    ) -> Result<Directory, RuntimeError> {
        super::deadline::ensure_setup(deadline)?;
        let directory = self.directory(parent)?;
        super::deadline::ensure_setup(deadline)?;
        Ok(directory)
    }

    fn directory_at_canonical_path<O: Operations>(
        &self,
        canonical: &Path,
        operations: &O,
    ) -> Result<Directory, RuntimeError> {
        let locked = self
            .directories
            .iter()
            .find(|value| value.path == canonical)
            .ok_or_else(|| RuntimeError::new("output directory changed after lock acquisition"))?;
        let current = open(
            canonical,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| failure(canonical, "reopen output directory", error))?;
        let expected = operations
            .before(FailurePoint::InspectLockedDirectory)
            .and_then(|()| rustix::fs::fstat(&locked.directory))
            .map_err(|error| failure(canonical, "inspect locked output directory", error))?;
        let actual = operations
            .before(FailurePoint::InspectCurrentDirectory)
            .and_then(|()| rustix::fs::fstat(current))
            .map_err(|error| failure(canonical, "inspect current output directory", error))?;
        if expected.st_dev != actual.st_dev || expected.st_ino != actual.st_ino {
            return Err(RuntimeError::new(
                "output directory changed after lock acquisition",
            ));
        }
        let descriptor = operations
            .before(FailurePoint::DuplicateDirectory)
            .and_then(|()| rustix::io::dup(&locked.directory))
            .map_err(|error| {
                RuntimeError::new(format!("cannot duplicate locked output directory: {error}"))
            })?;
        Ok(Directory::from_descriptor(
            canonical.to_path_buf(),
            descriptor,
        ))
    }
}

fn lock<O: Operations>(parent: &Path, operations: &O) -> Result<LockedDirectory, RuntimeError> {
    let directory = open(
        parent,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| failure(parent, "open output directory", error))?;
    let lock = openat(
        &directory,
        LOCK_NAME,
        OFlags::CREATE | OFlags::RDWR | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    )
    .map_err(|error| failure(parent, "open render lock", error))?;
    let metadata = operations
        .before(FailurePoint::InspectLock)
        .and_then(|()| rustix::fs::fstat(&lock))
        .map_err(|error| failure(parent, "inspect render lock", error))?;
    if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile {
        return Err(RuntimeError::new(format!(
            "render lock in {} must be a regular non-symlink file",
            parent.display()
        )));
    }
    operations
        .before(FailurePoint::AcquireLock)
        .and_then(|()| flock(&lock, FlockOperation::NonBlockingLockExclusive))
        .map_err(|error| {
            if error == Errno::WOULDBLOCK || error == Errno::AGAIN {
                RuntimeError::new(format!(
                    "output directory {} is locked by another VEAC render",
                    parent.display()
                ))
            } else {
                failure(parent, "acquire render lock", error)
            }
        })?;
    Ok(LockedDirectory {
        path: parent.to_path_buf(),
        directory,
        lock,
    })
}

fn failure(parent: &Path, action: &str, error: Errno) -> RuntimeError {
    RuntimeError::new(format!("cannot {action} in {}: {error}", parent.display()))
}

#[cfg(test)]
mod tests;
