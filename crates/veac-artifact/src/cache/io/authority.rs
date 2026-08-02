use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::{Path, PathBuf};

use rustix::fs::{fstat, mkdirat, openat, statat, AtFlags, FileType, Mode, OFlags};

use super::common::{corrupt, corrupt_io, io_error};
use super::path::{path_parts, target_parts};
use crate::ArtifactResult;

mod root;
mod symlink;

const DIRECTORY_FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::CLOEXEC);

#[derive(Debug)]
struct Node {
    name: OsString,
    file: File,
}

#[derive(Debug)]
pub(super) struct BoundChain {
    anchor: File,
    nodes: Vec<Node>,
    symlinks: Vec<symlink::SymlinkGuard>,
    path: PathBuf,
}

impl BoundChain {
    pub(super) fn extend(&mut self, names: &[OsString], create: bool) -> ArtifactResult<bool> {
        for name in names {
            if !self.push(name, create)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn current(&self) -> &File {
        self.nodes
            .last()
            .map(|node| &node.file)
            .unwrap_or(&self.anchor)
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn verify(&self) -> ArtifactResult<()> {
        for guard in &self.symlinks {
            guard.verify()?;
        }
        let mut parent = &self.anchor;
        for node in &self.nodes {
            let current = open_directory(parent, &node.name)
                .map_err(|error| corrupt_io("reopen bound cache directory", error))?;
            verify_identity(&node.file, &current, FileType::Directory)?;
            parent = &node.file;
        }
        Ok(())
    }

    pub(super) fn try_clone(&self) -> ArtifactResult<Self> {
        let anchor = self.anchor.try_clone()?;
        let nodes = self
            .nodes
            .iter()
            .map(|node| {
                Ok(Node {
                    name: node.name.clone(),
                    file: node.file.try_clone()?,
                })
            })
            .collect::<ArtifactResult<Vec<_>>>()?;
        let symlinks = self
            .symlinks
            .iter()
            .map(symlink::SymlinkGuard::try_clone)
            .collect::<ArtifactResult<Vec<_>>>()?;
        Ok(Self {
            anchor,
            nodes,
            symlinks,
            path: self.path.clone(),
        })
    }

    fn push(&mut self, name: &OsStr, create: bool) -> ArtifactResult<bool> {
        let file = match open_directory(self.current(), name) {
            Ok(file) => file,
            Err(rustix::io::Errno::NOENT) if !create => return Ok(false),
            Err(rustix::io::Errno::NOENT) => create_directory(self.current(), name)?,
            Err(error) => {
                return Err(corrupt_io(
                    &format!("open cache directory component {name:?}"),
                    error,
                ))
            }
        };
        self.append(name, file);
        Ok(true)
    }

    fn append(&mut self, name: &OsStr, file: File) {
        self.path.push(name);
        self.nodes.push(Node {
            name: name.to_owned(),
            file,
        });
    }
}

pub(super) fn target_parent(
    root: &Path,
    target: &Path,
    create: bool,
) -> ArtifactResult<Option<(BoundChain, OsString)>> {
    let (names, target_name) = target_parts(root, target)?;
    let Some(mut chain) = BoundChain::root(root, create)? else {
        return Ok(None);
    };
    if !chain.extend(&names, create)? {
        return Ok(None);
    }
    Ok(Some((chain, target_name)))
}

pub(super) fn verify_identity(
    expected: &File,
    current: &File,
    kind: FileType,
) -> ArtifactResult<()> {
    let expected = fstat(expected).map_err(|error| io_error("inspect bound cache entry", error))?;
    let current =
        fstat(current).map_err(|error| io_error("inspect reopened cache entry", error))?;
    if FileType::from_raw_mode(current.st_mode) != kind
        || expected.st_dev != current.st_dev
        || expected.st_ino != current.st_ino
    {
        return corrupt("cache directory identity changed during operation");
    }
    Ok(())
}

pub(super) fn directory_flags() -> OFlags {
    DIRECTORY_FLAGS
}

fn open_directory(parent: &File, name: &OsStr) -> rustix::io::Result<File> {
    openat(parent, name, DIRECTORY_FLAGS, Mode::empty()).map(File::from)
}

fn create_directory(parent: &File, name: &OsStr) -> ArtifactResult<File> {
    let created = match mkdirat(parent, name, Mode::RWXU) {
        Ok(()) => true,
        Err(rustix::io::Errno::EXIST) => false,
        Err(error) => return Err(io_error("create cache directory", error)),
    };
    let expected = if created {
        Some(
            statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|error| corrupt_io("bind created cache directory", error))?,
        )
    } else {
        None
    };
    let opened = open_directory(parent, name)
        .map_err(|error| corrupt_io("bind created cache directory", error))?;
    if let Some(expected) = expected {
        let actual = fstat(&opened).map_err(|error| io_error("inspect cache directory", error))?;
        if FileType::from_raw_mode(expected.st_mode) != FileType::Directory
            || expected.st_dev != actual.st_dev
            || expected.st_ino != actual.st_ino
        {
            return corrupt("created cache directory changed before it was opened");
        }
    }
    Ok(opened)
}

#[cfg(test)]
#[path = "authority/tests.rs"]
mod tests;
