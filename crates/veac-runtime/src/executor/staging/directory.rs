use std::path::{Path, PathBuf};

use rustix::fd::OwnedFd;
use rustix::fs::{fsync, mkdirat, open, openat, Mode, OFlags};
use rustix::io::Errno;

use crate::RuntimeError;

mod entry;
mod hash;
mod io;
mod list;
mod tree;

pub(in crate::executor) use entry::{EntryIdentity, EntryState};

#[derive(Debug)]
pub(in crate::executor) struct Directory {
    path: PathBuf,
    descriptor: OwnedFd,
}

impl Directory {
    pub(super) fn open(path: &Path) -> Result<Self, RuntimeError> {
        let descriptor = open(
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| failure(path, "open directory", error))?;
        Ok(Self::from_descriptor(path.to_path_buf(), descriptor))
    }

    pub(in crate::executor) fn from_descriptor(path: PathBuf, descriptor: OwnedFd) -> Self {
        Self { path, descriptor }
    }

    pub(super) fn create_child(&self, name: &str) -> Result<Self, RuntimeError> {
        mkdirat(&self.descriptor, name, Mode::RWXU)
            .map_err(|error| failure(&self.path, "create transaction directory", error))?;
        self.child(name)
    }

    pub(super) fn child(&self, name: &str) -> Result<Self, RuntimeError> {
        let descriptor = openat(
            &self.descriptor,
            name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| failure(&self.path, "open transaction directory", error))?;
        Ok(Self::from_descriptor(self.path.join(name), descriptor))
    }

    pub(super) fn sync(&self) -> Result<(), RuntimeError> {
        fsync(&self.descriptor).map_err(|error| failure(&self.path, "sync directory", error))
    }
}

pub(super) fn failure(parent: &Path, action: &str, error: Errno) -> RuntimeError {
    RuntimeError::new(format!(
        "atomic render output commit failed: cannot {action} in {}: {error}",
        parent.display()
    ))
}
