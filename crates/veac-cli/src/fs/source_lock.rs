use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{flock, open, openat, FileType, FlockOperation, Mode, OFlags};
use rustix::io::Errno;

use crate::error::{CliError, CliResult};

macro_rules! cli_try {
    ($result:expr, |$error:ident| $mapped:expr) => {
        match $result {
            Ok(value) => value,
            Err($error) => return Err($mapped),
        }
    };
}

mod commit;
mod path;
mod stage;

pub(crate) const SOURCE_LOCK_NAME: &str = ".veac-source.lock";

#[derive(Debug)]
pub(crate) struct SourceGraphLock {
    pub(super) directory: OwnedFd,
    lock: OwnedFd,
}

impl SourceGraphLock {
    pub(crate) fn acquire(root: &Path) -> CliResult<Self> {
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
        let metadata = cli_try!(rustix::fs::fstat(&lock), |error| failure(
            root,
            "inspect source lock",
            error
        ));
        if FileType::from_raw_mode(metadata.st_mode) != FileType::RegularFile {
            return Err(CliError::new(
                "SOURCE_LOCK_FAILED",
                format!("source lock in {} is not a regular file", root.display()),
            ));
        }
        flock(&lock, FlockOperation::NonBlockingLockExclusive).map_err(|error| {
            if error == Errno::WOULDBLOCK {
                CliError::new(
                    "SOURCE_LOCKED",
                    format!("source graph {} is being edited", root.display()),
                )
            } else {
                failure(root, "acquire source lock", error)
            }
        })?;
        Ok(Self { directory, lock })
    }

    pub(crate) fn revalidate(&self, root: &Path) -> CliResult {
        let current = open(
            root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| changed(root, "source root is unavailable", error))?;
        let expected = cli_try!(rustix::fs::fstat(&self.directory), |error| changed(
            root,
            "cannot inspect locked source root",
            error
        ));
        let actual = cli_try!(rustix::fs::fstat(current), |error| changed(
            root,
            "cannot inspect current source root",
            error
        ));
        require_identity(root, "source root", &expected, &actual)?;
        self.revalidate_lock(root)
    }

    fn revalidate_lock(&self, root: &Path) -> CliResult {
        let current = openat(
            &self.directory,
            SOURCE_LOCK_NAME,
            OFlags::RDWR | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| changed(root, "source lock path is unavailable", error))?;
        let expected = cli_try!(rustix::fs::fstat(&self.lock), |error| changed(
            root,
            "cannot inspect held source lock",
            error
        ));
        let actual = cli_try!(rustix::fs::fstat(current), |error| changed(
            root,
            "cannot inspect current source lock",
            error
        ));
        require_identity(root, "source lock", &expected, &actual)
    }
}

impl Drop for SourceGraphLock {
    fn drop(&mut self) {
        let _ = flock(&self.lock, FlockOperation::Unlock);
    }
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
        format!("{action} {} during the edit: {error}", root.display()),
    )
}

fn require_identity(
    root: &Path,
    role: &str,
    expected: &rustix::fs::Stat,
    actual: &rustix::fs::Stat,
) -> CliResult {
    if expected.st_dev == actual.st_dev && expected.st_ino == actual.st_ino {
        return Ok(());
    }
    Err(CliError::new(
        "SOURCE_CHANGED",
        format!("{role} {} changed during the edit", root.display()),
    ))
}

#[cfg(test)]
#[path = "source_lock/tests.rs"]
mod tests;
