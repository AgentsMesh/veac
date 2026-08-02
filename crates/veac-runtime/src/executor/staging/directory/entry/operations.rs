use rustix::fs::{fstat, fsync, openat, renameat, statat, unlinkat, AtFlags, Mode, OFlags};
use rustix::io::Errno;

use super::identity::{from_stat, identity};
use super::model::{EntryIdentity, EntryState, RenameFailure};
use crate::executor::staging::directory::{failure, Directory};
use crate::RuntimeError;

impl Directory {
    pub(in crate::executor) fn state(&self, name: &str) -> Result<EntryState, RuntimeError> {
        match statat(&self.descriptor, name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(metadata) => identity(&self.path, name, metadata).map(EntryState::from_identity),
            Err(Errno::NOENT) => Ok(EntryState::Missing),
            Err(error) => Err(failure(&self.path, "inspect transaction path", error)),
        }
    }

    pub(in crate::executor) fn require(
        &self,
        name: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError> {
        if self.state(name)?.matches(expected) {
            Ok(())
        } else {
            Err(RuntimeError::new(format!(
                "descriptor-relative path {} changed identity",
                self.path.join(name).display()
            )))
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
        renameat(&self.descriptor, source, &target.descriptor, destination)
            .map_err(|error| not_crossed(failure(&self.path, "rename transaction path", error)))?;
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
        unlinkat(&self.descriptor, name, expected.remove_flags())
            .map_err(|error| failure(&self.path, "remove transaction path", error))
    }

    pub(in crate::executor) fn sync_bound(
        &self,
        name: &str,
        expected: EntryIdentity,
    ) -> Result<(), RuntimeError> {
        self.require(name, expected)?;
        let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        let file = openat(&self.descriptor, name, flags, Mode::empty())
            .map_err(|error| failure(&self.path, "open transaction output", error))?;
        let metadata = fstat(&file)
            .map_err(|error| failure(&self.path, "inspect opened transaction output", error))?;
        if from_stat(metadata)? != expected {
            return Err(RuntimeError::new(
                "opened transaction output changed identity",
            ));
        }
        fsync(file).map_err(|error| failure(&self.path, "sync transaction output", error))
    }
}

fn not_crossed(error: RuntimeError) -> RenameFailure {
    RenameFailure {
        error,
        crossed_commit: false,
    }
}
