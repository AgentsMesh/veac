use rustix::fs::{fstat, openat, renameat, statat, unlinkat, AtFlags, FileType, Mode, OFlags};
use rustix::io::Errno;
use serde::{Deserialize, Serialize};

use super::{failure, Directory};
use crate::RuntimeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::executor) struct EntryIdentity {
    pub device: u64,
    pub inode: u64,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::executor) enum EntryState {
    Missing,
    Regular(EntryIdentity),
}

#[derive(Debug)]
pub(in crate::executor) struct RenameFailure {
    pub error: RuntimeError,
    pub crossed_commit: bool,
}

impl Directory {
    pub(in crate::executor) fn state(&self, name: &str) -> Result<EntryState, RuntimeError> {
        match statat(&self.descriptor, name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(metadata) => identity(&self.path, name, metadata).map(EntryState::Regular),
            Err(Errno::NOENT) => Ok(EntryState::Missing),
            Err(error) => Err(failure(&self.path, "inspect transaction path", error)),
        }
    }

    pub(in crate::executor) fn require(
        &self,
        name: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError> {
        match self.state(name)? {
            EntryState::Regular(actual) if actual == expected => Ok(()),
            _ => Err(RuntimeError::new(format!(
                "descriptor-relative path {} changed identity",
                self.path.join(name).display()
            ))),
        }
    }

    pub(in crate::executor) fn rename_bound_to(
        &self,
        source: &str,
        expected: EntryIdentity,
        target: &Directory,
        destination: &str,
    ) -> Result<(), RenameFailure> {
        self.rename_bound_to_with(source, expected, target, destination, || {})
    }

    pub(in crate::executor) fn rename_bound_to_with(
        &self,
        source: &str,
        expected: EntryIdentity,
        target: &Directory,
        destination: &str,
        before_rename: impl FnOnce(),
    ) -> Result<(), RenameFailure> {
        self.require(source, expected).map_err(not_crossed)?;
        before_rename();
        if let Err(error) = renameat(&self.descriptor, source, &target.descriptor, destination) {
            return Err(not_crossed(failure(
                &self.path,
                "rename transaction path",
                error,
            )));
        }
        target
            .require(destination, expected)
            .map_err(|error| RenameFailure {
                error,
                crossed_commit: true,
            })
    }

    pub(in crate::executor) fn remove_bound(
        &self,
        name: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError> {
        self.require(name, expected)?;
        unlinkat(&self.descriptor, name, AtFlags::empty())
            .map_err(|error| failure(&self.path, "remove transaction path", error))
    }

    pub(in crate::executor) fn sync_bound(
        &self,
        name: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError> {
        self.require(name, expected)?;
        let file = openat(
            &self.descriptor,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| failure(&self.path, "open transaction output", error))?;
        let metadata = match fstat(&file) {
            Ok(metadata) => metadata,
            Err(error) => {
                return Err(failure(
                    &self.path,
                    "inspect opened transaction output",
                    error,
                ))
            }
        };
        let actual = from_stat(metadata)?;
        if actual != expected {
            return Err(RuntimeError::new(
                "opened transaction output changed identity",
            ));
        }
        match rustix::fs::fsync(file) {
            Ok(()) => Ok(()),
            Err(error) => Err(failure(&self.path, "sync transaction output", error)),
        }
    }
}

fn identity(
    parent: &std::path::Path,
    name: &str,
    metadata: rustix::fs::Stat,
) -> Result<EntryIdentity, RuntimeError> {
    if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile || metadata.st_nlink != 1
    {
        return Err(RuntimeError::new(format!(
            "descriptor-relative path {} must be an exclusively linked regular file",
            parent.join(name).display()
        )));
    }
    from_stat(metadata)
}

fn from_stat(metadata: rustix::fs::Stat) -> Result<EntryIdentity, RuntimeError> {
    Ok(EntryIdentity {
        device: identity_number(metadata.st_dev, "device number")?,
        inode: identity_number(metadata.st_ino, "inode number")?,
        size_bytes: nonnegative_size(metadata.st_size)?,
    })
}

fn identity_number<T: TryInto<u64>>(value: T, name: &str) -> Result<u64, RuntimeError> {
    match value.try_into() {
        Ok(value) => Ok(value),
        Err(_) => Err(RuntimeError::new(format!(
            "descriptor-relative {name} is out of range"
        ))),
    }
}

fn nonnegative_size<T: TryInto<u64>>(value: T) -> Result<u64, RuntimeError> {
    match value.try_into() {
        Ok(value) => Ok(value),
        Err(_) => Err(RuntimeError::new(
            "descriptor-relative file size is negative",
        )),
    }
}

pub(super) fn opened_identity(file: &std::fs::File) -> Result<EntryIdentity, RuntimeError> {
    let metadata = match fstat(file) {
        Ok(metadata) => metadata,
        Err(error) => {
            return Err(RuntimeError::new(format!(
                "cannot inspect opened transaction file: {error}"
            )))
        }
    };
    if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile || metadata.st_nlink != 1
    {
        return Err(RuntimeError::new(
            "opened transaction file must be an exclusively linked regular file",
        ));
    }
    from_stat(metadata)
}

fn not_crossed(error: RuntimeError) -> RenameFailure {
    RenameFailure {
        error,
        crossed_commit: false,
    }
}
#[cfg(test)]
#[path = "entry/tests.rs"]
mod tests;
