use std::ffi::OsString;
use std::fs::File;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use rustix::fs::{fstat, mkdirat, openat, statat, unlinkat, AtFlags, Dir, FileType, Mode};

use super::authority::{directory_flags, verify_identity, BoundChain};
use super::common::{corrupt, corrupt_io, io_error, resource_limit};
use crate::ArtifactResult;

#[derive(Debug)]
pub(super) struct BoundDirectory {
    parent: BoundChain,
    name: OsString,
    file: File,
    path: PathBuf,
}

impl BoundDirectory {
    pub(super) fn open(parent: BoundChain, name: OsString) -> ArtifactResult<Option<Self>> {
        let file = match openat(parent.current(), &name, directory_flags(), Mode::empty()) {
            Ok(file) => File::from(file),
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(error) => return Err(corrupt_io("open artifact directory", error)),
        };
        let path = parent.path().join(&name);
        let directory = Self {
            parent,
            name,
            file,
            path,
        };
        directory.verify()?;
        Ok(Some(directory))
    }

    pub(super) fn create(parent: BoundChain, name: OsString) -> ArtifactResult<Self> {
        mkdirat(parent.current(), &name, Mode::RWXU)
            .map_err(|error| io_error("create private cache directory", error))?;
        let expected = statat(parent.current(), &name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| corrupt_io("bind private cache directory", error))?;
        let file = openat(parent.current(), &name, directory_flags(), Mode::empty())
            .map(File::from)
            .map_err(|error| corrupt_io("open private cache directory", error))?;
        let actual =
            fstat(&file).map_err(|error| io_error("inspect private cache directory", error))?;
        if FileType::from_raw_mode(expected.st_mode) != FileType::Directory
            || expected.st_dev != actual.st_dev
            || expected.st_ino != actual.st_ino
        {
            return corrupt("created cache directory changed before it was opened");
        }
        let path = parent.path().join(&name);
        Ok(Self {
            parent,
            name,
            file,
            path,
        })
    }

    pub(super) fn file(&self) -> &File {
        &self.file
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn verify(&self) -> ArtifactResult<()> {
        self.parent.verify()?;
        let current = openat(
            self.parent.current(),
            &self.name,
            directory_flags(),
            Mode::empty(),
        )
        .map(File::from)
        .map_err(|error| corrupt_io("reopen artifact directory", error))?;
        verify_identity(&self.file, &current, FileType::Directory)
    }

    pub(super) fn entries(&self, limit: usize) -> ArtifactResult<Vec<OsString>> {
        directory_entries(&self.file, limit)
    }

    pub(super) fn sync(&self) -> ArtifactResult<()> {
        self.file.sync_all()?;
        Ok(())
    }

    pub(super) fn sync_parent(&self) -> ArtifactResult<()> {
        self.parent.current().sync_all()?;
        Ok(())
    }

    pub(super) fn remove_empty(&self) -> ArtifactResult<()> {
        self.verify()?;
        unlinkat(self.parent.current(), &self.name, AtFlags::REMOVEDIR)
            .map_err(|error| io_error("remove cache directory", error))
    }

    pub(super) fn remove_empty_into_parent(self) -> ArtifactResult<(BoundChain, OsString)> {
        self.remove_empty()?;
        self.parent.current().sync_all()?;
        Ok((self.parent, self.name))
    }
}

pub(super) fn directory_entries(file: &File, limit: usize) -> ArtifactResult<Vec<OsString>> {
    let mut names = Vec::new();
    let entries =
        Dir::read_from(file).map_err(|error| io_error("read artifact directory", error))?;
    for entry in entries {
        let entry = entry.map_err(|error| io_error("read artifact directory entry", error))?;
        let bytes = entry.file_name().to_bytes();
        if bytes == b"." || bytes == b".." {
            continue;
        }
        if names.len() == limit {
            return resource_limit("cache directory entry count exceeds the limit");
        }
        names.push(OsString::from_vec(bytes.to_vec()));
    }
    names.sort();
    Ok(names)
}

#[cfg(test)]
#[path = "directory/tests.rs"]
mod tests;
