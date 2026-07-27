use std::fs::File;

use rustix::fs::{fstat, FileType, Stat};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

pub(super) fn inspect(file: &File, kind: FileType) -> ArtifactResult<Stat> {
    let state = fstat(file).map_err(io_error)?;
    if FileType::from_raw_mode(state.st_mode) == kind {
        Ok(state)
    } else {
        unsafe_path("relink entry type changed during discovery")
    }
}

pub(super) fn require_same(expected: &Stat, current: &Stat) -> ArtifactResult<()> {
    if same_state(expected, current) {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::IdentityMismatch,
            "relink search entry changed during discovery",
        ))
    }
}

fn same_state(left: &Stat, right: &Stat) -> bool {
    left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
        && left.st_size == right.st_size
        && left.st_mtime == right.st_mtime
        && left.st_mtime_nsec == right.st_mtime_nsec
        && left.st_ctime == right.st_ctime
        && left.st_ctime_nsec == right.st_ctime_nsec
}

pub(super) fn open_error(error: rustix::io::Errno) -> ArtifactError {
    if matches!(error, rustix::io::Errno::LOOP | rustix::io::Errno::NOTDIR) {
        ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "relink search trees may not contain symlinks",
        )
    } else {
        io_error(error)
    }
}

pub(super) fn io_error(error: rustix::io::Errno) -> ArtifactError {
    std::io::Error::from_raw_os_error(error.raw_os_error()).into()
}

pub(super) fn unsafe_path<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(ArtifactErrorKind::UnsafePath, message))
}

#[cfg(test)]
#[path = "state/tests.rs"]
mod tests;
