use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{flock, open, openat, FileType, FlockOperation, Mode, OFlags};
use rustix::io::Errno;

use crate::error::{CliError, CliResult};

use super::SOURCE_LOCK_NAME;

pub(super) fn acquire(
    root: &Path,
    operation: FlockOperation,
    blocked_activity: &str,
) -> CliResult<(OwnedFd, OwnedFd)> {
    let directory = open(
        root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| failure(root, "open source root", error))?;
    let lock = openat(
        &directory,
        SOURCE_LOCK_NAME,
        OFlags::CREATE | OFlags::RDWR | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    )
    .map_err(|error| failure(root, "open source lock", error))?;
    let metadata =
        rustix::fs::fstat(&lock).map_err(|error| failure(root, "inspect source lock", error))?;
    if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile {
        return Err(CliError::new(
            "SOURCE_LOCK_FAILED",
            format!("source lock in {} is not a regular file", root.display()),
        ));
    }
    flock(&lock, operation).map_err(|error| {
        if error == Errno::WOULDBLOCK {
            CliError::new(
                "SOURCE_LOCKED",
                format!(
                    "source graph {} is being {blocked_activity}",
                    root.display()
                ),
            )
        } else {
            failure(root, "acquire source lock", error)
        }
    })?;
    Ok((directory, lock))
}

pub(super) fn revalidate(root: &Path, directory: &OwnedFd, lock: &OwnedFd) -> CliResult {
    let current = open(
        root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| changed(root, "source root is unavailable", error))?;
    require_same_file(
        root,
        "source root",
        rustix::fs::fstat(directory),
        rustix::fs::fstat(current),
    )?;
    let current = openat(
        directory,
        SOURCE_LOCK_NAME,
        OFlags::RDWR | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| changed(root, "source lock path is unavailable", error))?;
    require_same_file(
        root,
        "source lock",
        rustix::fs::fstat(lock),
        rustix::fs::fstat(current),
    )
}

pub(super) fn unlock(lock: &OwnedFd) {
    let _ = flock(lock, FlockOperation::Unlock);
}

fn require_same_file(
    root: &Path,
    role: &str,
    expected: Result<rustix::fs::Stat, Errno>,
    actual: Result<rustix::fs::Stat, Errno>,
) -> CliResult {
    let expected = expected.map_err(|error| changed(root, "cannot inspect held file", error))?;
    let actual = actual.map_err(|error| changed(root, "cannot inspect current file", error))?;
    if expected.st_dev == actual.st_dev && expected.st_ino == actual.st_ino {
        return Ok(());
    }
    Err(CliError::new(
        "SOURCE_CHANGED",
        format!("{role} {} changed while locked", root.display()),
    ))
}

fn failure(root: &Path, action: &str, error: Errno) -> CliError {
    CliError::new(
        "SOURCE_LOCK_FAILED",
        format!("cannot {action} {}: {error}", root.display()),
    )
}

fn changed(root: &Path, action: &str, error: Errno) -> CliError {
    CliError::new(
        "SOURCE_CHANGED",
        format!("{action} {} while locked: {error}", root.display()),
    )
}
