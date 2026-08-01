use std::fs::File;
use std::io::{Read, Write};

use super::entry::opened_identity;
use super::{failure, Directory, EntryIdentity, EntryState};
use crate::RuntimeError;
use rustix::fs::{openat, Mode, OFlags};

impl Directory {
    pub(in crate::executor) fn create_regular(&self, name: &str) -> Result<File, RuntimeError> {
        let flags =
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        openat(&self.descriptor, name, flags, Mode::RUSR | Mode::WUSR)
            .map(File::from)
            .map_err(|error| failure(&self.path, "create transaction file", error))
    }

    pub(in crate::executor) fn read_bounded(
        &self,
        name: &str,
        maximum: u64,
    ) -> Result<(Vec<u8>, EntryIdentity), RuntimeError> {
        let expected = match self.state(name)? {
            EntryState::Regular(identity) => identity,
            EntryState::Directory(_) => {
                return Err(RuntimeError::new(
                    "descriptor-relative bounded read requires a regular file",
                ))
            }
            EntryState::Missing => {
                return Err(RuntimeError::new("descriptor-relative file is missing"))
            }
        };
        let size_bytes = expected.size_bytes().expect("regular identity has a size");
        if size_bytes > maximum {
            return Err(RuntimeError::new(
                "descriptor-relative file exceeds its bounded read limit",
            ));
        }
        let file = match openat(
            &self.descriptor,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        ) {
            Ok(file) => File::from(file),
            Err(error) => return Err(failure(&self.path, "open transaction file", error)),
        };
        if opened_identity(&file)? != expected {
            return Err(RuntimeError::new(
                "opened transaction file changed identity before bounded read",
            ));
        }
        let capacity = match usize::try_from(size_bytes) {
            Ok(capacity) => capacity,
            Err(_) => {
                return Err(RuntimeError::new(
                    "descriptor-relative file is too large to read",
                ))
            }
        };
        let mut reader = file.take(maximum.saturating_add(1));
        let mut bytes = Vec::with_capacity(capacity);
        if let Err(error) = reader.read_to_end(&mut bytes) {
            return Err(RuntimeError::new(format!(
                "cannot read transaction file: {error}"
            )));
        }
        if bytes.len() as u64 != size_bytes {
            return Err(RuntimeError::new(
                "descriptor-relative file changed while it was read",
            ));
        }
        self.require(name, expected)?;
        Ok((bytes, expected))
    }

    pub(in crate::executor) fn write_all_sync(
        &self,
        name: &str,
        bytes: &[u8],
    ) -> Result<EntryIdentity, RuntimeError> {
        let mut file = self.create_regular(name)?;
        if let Err(error) = file.write_all(bytes) {
            return Err(RuntimeError::new(format!(
                "cannot write transaction file: {error}"
            )));
        }
        if let Err(error) = file.sync_all() {
            return Err(RuntimeError::new(format!(
                "cannot sync transaction file: {error}"
            )));
        }
        let expected = opened_identity(&file)?;
        self.require(name, expected)?;
        drop(file);
        Ok(expected)
    }

    pub(in crate::executor) fn missing(&self, name: &str) -> Result<bool, RuntimeError> {
        Ok(matches!(self.state(name)?, EntryState::Missing))
    }

    pub(in crate::executor) fn remove_if_regular(&self, name: &str) -> Result<(), RuntimeError> {
        match self.state(name) {
            Ok(EntryState::Missing) => Ok(()),
            Ok(EntryState::Regular(identity)) => self.remove_bound(name, identity),
            Ok(EntryState::Directory(_)) => Err(RuntimeError::new(
                "descriptor-relative path must be a regular file",
            )),
            Err(error) => Err(error),
        }
    }
}
