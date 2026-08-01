use std::ffi::{OsStr, OsString};
use std::fs::File;

use rustix::fs::{fstat, mkdirat, openat, statat, unlinkat, AtFlags, FileType, Mode};

use super::{directory_flags, io_error, unsafe_io, unsafe_path, verify_identity};
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

pub(in crate::staged) fn create_private_directory(
    parent: &File,
) -> ArtifactResult<(OsString, File)> {
    create_private_directory_with(parent, |_| {})
}

fn create_private_directory_with(
    parent: &File,
    after_create: impl FnOnce(&OsStr),
) -> ArtifactResult<(OsString, File)> {
    let mut after_create = Some(after_create);
    for _ in 0..128 {
        let name = random_name()?;
        match mkdirat(parent, &name, Mode::RWXU) {
            Ok(()) => {
                let created = CreatedDirectory::new(parent, name)?;
                after_create.take().expect("creation observer runs once")(&created.name);
                return created.open();
            }
            Err(rustix::io::Errno::EXIST) => continue,
            Err(error) => return Err(io_error("create private directory", error)),
        }
    }
    Err(unsafe_path("cannot allocate a unique staging directory"))
}

struct CreatedDirectory<'a> {
    parent: &'a File,
    name: OsString,
    identity: Option<File>,
    armed: bool,
}

impl<'a> CreatedDirectory<'a> {
    fn new(parent: &'a File, name: OsString) -> ArtifactResult<Self> {
        let identity = openat(parent, &name, directory_flags(), Mode::empty())
            .map(File::from)
            .map_err(|error| unsafe_io("bind created private directory", error))?;
        Ok(Self {
            parent,
            name,
            identity: Some(identity),
            armed: true,
        })
    }

    fn open(mut self) -> ArtifactResult<(OsString, File)> {
        let opened = openat(self.parent, &self.name, directory_flags(), Mode::empty())
            .map(File::from)
            .map_err(|error| unsafe_io("open created private directory", error));
        let directory = match opened {
            Ok(directory) => directory,
            Err(primary) => return Err(self.cleanup_after(primary)),
        };
        let expected = self
            .identity
            .as_ref()
            .ok_or_else(|| unsafe_path("created staging directory has no bound open identity"));
        match expected
            .and_then(|expected| verify_identity(expected, &directory, FileType::Directory))
        {
            Ok(()) => {
                self.armed = false;
                Ok((self.name.clone(), directory))
            }
            Err(primary) => Err(self.cleanup_after(primary)),
        }
    }

    fn cleanup_after(&mut self, primary: ArtifactError) -> ArtifactError {
        match self.cleanup() {
            Ok(()) => primary,
            Err(cleanup) => primary.with_cleanup_failure(
                cleanup,
                "private staging directory creation failed and cleanup was unsafe",
            ),
        }
    }

    fn cleanup(&mut self) -> ArtifactResult<()> {
        if !self.armed {
            return Ok(());
        }
        let identity = self.identity.as_ref().ok_or_else(|| {
            unsafe_path("created staging directory has no bound cleanup identity")
        })?;
        let expected =
            fstat(identity).map_err(|error| unsafe_io("inspect bound created directory", error))?;
        let current = match statat(self.parent, &self.name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(current) => current,
            Err(rustix::io::Errno::NOENT) => {
                self.armed = false;
                return Ok(());
            }
            Err(error) => return Err(unsafe_io("inspect created directory for cleanup", error)),
        };
        if expected.st_dev != current.st_dev
            || expected.st_ino != current.st_ino
            || FileType::from_raw_mode(current.st_mode) != FileType::Directory
        {
            return Err(unsafe_path(
                "created staging directory changed before cleanup",
            ));
        }
        unlinkat(self.parent, &self.name, AtFlags::REMOVEDIR)
            .map_err(|error| io_error("remove created private directory", error))?;
        self.armed = false;
        Ok(())
    }
}

impl Drop for CreatedDirectory<'_> {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

fn random_name() -> ArtifactResult<OsString> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|error| {
        ArtifactError::with_source(
            ArtifactErrorKind::Io,
            "staged output cannot read operating-system randomness",
            error,
        )
    })?;
    let mut name = String::from(".veac-stage-");
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        name.push(HEX[(byte >> 4) as usize] as char);
        name.push(HEX[(byte & 0x0f) as usize] as char);
    }
    Ok(name.into())
}

#[cfg(test)]
mod tests;
