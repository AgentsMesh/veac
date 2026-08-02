use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use rustix::fs::{fstat, openat, Dir, FileType, Mode, OFlags, Stat};

use crate::{ArtifactResult, VerifiedSourceCopy};

use super::super::DiscoveryBudget;

#[path = "unix/root.rs"]
mod root;
#[path = "unix/state.rs"]
mod state;
pub(crate) use root::*;
use state::*;

const ENTRY_FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);
pub(super) const DIRECTORY_FLAGS: OFlags = ENTRY_FLAGS.union(OFlags::DIRECTORY);

pub(crate) struct BoundDirectory {
    file: File,
    path: PathBuf,
    opened: Stat,
}

pub(crate) struct BoundFile {
    file: File,
    path: PathBuf,
    opened: Stat,
}

pub(crate) enum BoundEntry {
    Directory(BoundDirectory),
    File(BoundFile),
}

pub(crate) struct EntrySeal {
    state: Stat,
    kind: FileType,
}

impl BoundDirectory {
    pub(crate) fn new(file: File, path: PathBuf) -> ArtifactResult<Self> {
        let opened = inspect(&file, FileType::Directory)?;
        Ok(Self { file, path, opened })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn entries(&self, budget: &mut DiscoveryBudget) -> ArtifactResult<Vec<OsString>> {
        budget.check_deadline()?;
        let entries = Dir::read_from(&self.file).map_err(io_error)?;
        let mut names = Vec::new();
        for entry in entries {
            budget.check_deadline()?;
            let entry = entry.map_err(io_error)?;
            let bytes = entry.file_name().to_bytes();
            if bytes == b"." || bytes == b".." {
                continue;
            }
            budget.observe_entry()?;
            names.push(OsString::from_vec(bytes.to_vec()));
        }
        names.sort();
        budget.check_deadline()?;
        Ok(names)
    }

    pub(crate) fn open_entry(
        &self,
        name: &OsStr,
        budget: &DiscoveryBudget,
    ) -> ArtifactResult<BoundEntry> {
        budget.check_deadline()?;
        let file = openat(&self.file, name, ENTRY_FLAGS, Mode::empty())
            .map(File::from)
            .map_err(open_error)?;
        budget.check_deadline()?;
        let state = fstat(&file).map_err(io_error)?;
        budget.check_deadline()?;
        let path = self.path.join(name);
        match FileType::from_raw_mode(state.st_mode) {
            FileType::Directory => Ok(BoundEntry::Directory(BoundDirectory {
                file,
                path,
                opened: state,
            })),
            FileType::RegularFile => Ok(BoundEntry::File(BoundFile {
                file,
                path,
                opened: state,
            })),
            _ => unsafe_path("relink search trees may contain only files and directories"),
        }
    }

    pub(crate) fn seal(&self, budget: &DiscoveryBudget) -> ArtifactResult<EntrySeal> {
        budget.check_deadline()?;
        let state = inspect(&self.file, FileType::Directory)?;
        budget.check_deadline()?;
        require_same(&self.opened, &state)?;
        Ok(EntrySeal {
            state,
            kind: FileType::Directory,
        })
    }

    pub(crate) fn verify_entry(
        &self,
        name: &OsStr,
        seal: &EntrySeal,
        budget: &DiscoveryBudget,
    ) -> ArtifactResult<()> {
        budget.check_deadline()?;
        let current = openat(&self.file, name, ENTRY_FLAGS, Mode::empty())
            .map(File::from)
            .map_err(open_error)?;
        budget.check_deadline()?;
        let state = inspect(&current, seal.kind)?;
        budget.check_deadline()?;
        require_same(&seal.state, &state)
    }
}

impl BoundFile {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn verify(
        &mut self,
        budget: &DiscoveryBudget,
    ) -> ArtifactResult<(VerifiedSourceCopy, EntrySeal)> {
        let limit = budget.file_limit()?;
        let verified =
            crate::source::verify_open_source_bounded_while(&mut self.file, None, limit, || {
                budget.before_deadline()
            })?;
        budget.check_deadline()?;
        let state = inspect(&self.file, FileType::RegularFile)?;
        budget.check_deadline()?;
        require_same(&self.opened, &state)?;
        Ok((
            verified,
            EntrySeal {
                state,
                kind: FileType::RegularFile,
            },
        ))
    }
}

fn unsafe_path<T>(message: &str) -> ArtifactResult<T> {
    state::unsafe_path(message)
}
