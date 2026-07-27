use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path};

use rustix::fs::{readlinkat, statat, AtFlags, FileType, Stat};

use crate::cache::io::common::{corrupt, corrupt_io, unsafe_path};
use crate::ArtifactResult;

#[derive(Debug)]
pub(super) struct SymlinkGuard {
    parent: File,
    name: OsString,
    identity: Stat,
}

impl SymlinkGuard {
    pub(super) fn verify(&self) -> ArtifactResult<()> {
        let current = statat(&self.parent, &self.name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| corrupt_io("reinspect cache root symlink", error))?;
        if !same_link(&self.identity, &current) {
            return corrupt("cache root symlink identity changed during operation");
        }
        Ok(())
    }

    pub(super) fn try_clone(&self) -> ArtifactResult<Self> {
        Ok(Self {
            parent: self.parent.try_clone()?,
            name: self.name.clone(),
            identity: self.identity,
        })
    }
}

pub(super) fn resolve(
    parent: &File,
    name: &OsStr,
) -> ArtifactResult<Option<(SymlinkGuard, Vec<OsString>)>> {
    let before = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| corrupt_io("inspect cache root symlink", error))?;
    if FileType::from_raw_mode(before.st_mode) != FileType::Symlink {
        return Ok(None);
    }
    let target = readlinkat(parent, name, Vec::new())
        .map_err(|error| corrupt_io("read cache root symlink", error))?;
    let after = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| corrupt_io("reinspect cache root symlink", error))?;
    if !same_link(&before, &after) {
        return corrupt("cache root symlink changed while it was resolved");
    }
    let names = relative_names(Path::new(OsStr::from_bytes(target.to_bytes())))?;
    Ok(Some((
        SymlinkGuard {
            parent: parent.try_clone()?,
            name: name.to_owned(),
            identity: before,
        },
        names,
    )))
}

fn relative_names(path: &Path) -> ArtifactResult<Vec<OsString>> {
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(name) => names.push(name.to_owned()),
            Component::RootDir | Component::ParentDir | Component::Prefix(_) => {
                return unsafe_path("cache root symlink target must stay relative to its parent")
            }
        }
    }
    if names.is_empty() {
        return unsafe_path("cache root symlink target cannot be empty");
    }
    Ok(names)
}

fn same_link(left: &Stat, right: &Stat) -> bool {
    FileType::from_raw_mode(right.st_mode) == FileType::Symlink
        && left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
}

#[cfg(test)]
#[path = "symlink/tests.rs"]
mod tests;
