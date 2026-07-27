use std::fs::File;
use std::path::Path;

use rustix::fs::{fstat, open, FileType, Mode, OFlags};

use super::PublishMode;
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

mod create;

pub(super) use create::create_private_directory;

pub(super) fn open_directory(path: &Path) -> ArtifactResult<File> {
    open(path, directory_flags(), Mode::empty())
        .map(File::from)
        .map_err(|error| io_error("open parent directory", error))
}

pub(super) fn directory_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}

pub(super) fn file_flags() -> OFlags {
    OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC
}

pub(super) fn verify_identity(
    expected: &File,
    current: &File,
    kind: FileType,
) -> ArtifactResult<()> {
    let expected = fstat(expected).map_err(|error| io_error("inspect opened entry", error))?;
    let current = fstat(current).map_err(|error| io_error("inspect reopened entry", error))?;
    if FileType::from_raw_mode(current.st_mode) != kind
        || expected.st_dev != current.st_dev
        || expected.st_ino != current.st_ino
    {
        return Err(unsafe_path(
            "filesystem entry no longer has its opened identity",
        ));
    }
    Ok(())
}

pub(super) fn verify_owned_regular(expected: &File, current: &File) -> ArtifactResult<()> {
    verify_identity(expected, current, FileType::RegularFile)?;
    let expected = fstat(expected).map_err(|error| io_error("inspect opened entry", error))?;
    let current = fstat(current).map_err(|error| io_error("inspect reopened entry", error))?;
    if expected.st_nlink != 1 || current.st_nlink != 1 {
        return Err(unsafe_path(
            "staged payload must have exactly one filesystem link",
        ));
    }
    Ok(())
}

pub(super) fn publish_error(mode: PublishMode, error: rustix::io::Errno) -> ArtifactError {
    if matches!(mode, PublishMode::NoClobber)
        && matches!(
            error,
            rustix::io::Errno::NOSYS
                | rustix::io::Errno::INVAL
                | rustix::io::Errno::NOTSUP
                | rustix::io::Errno::OPNOTSUPP
        )
    {
        return unsafe_io("atomic no-clobber publication is unsupported", error);
    }
    io_error("publish staged payload", error)
}

pub(super) fn io_error(operation: &str, error: rustix::io::Errno) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::Io,
        format!("staged output cannot {operation}"),
        std::io::Error::from_raw_os_error(error.raw_os_error()),
    )
}

pub(super) fn unsafe_io(operation: &str, error: rustix::io::Errno) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::UnsafePath,
        format!("staged output cannot safely {operation}"),
        std::io::Error::from_raw_os_error(error.raw_os_error()),
    )
}

pub(super) fn unsafe_path(message: &str) -> ArtifactError {
    ArtifactError::new(ArtifactErrorKind::UnsafePath, message)
}

#[cfg(test)]
#[path = "checks/tests.rs"]
mod tests;
