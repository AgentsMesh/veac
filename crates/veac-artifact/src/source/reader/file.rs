use std::fs::File;
use std::path::Path;

use rustix::fs::{fstat, open, FileType, Mode, OFlags, Stat};
use veac_ir::{HashAlgorithm, MediaIdentity};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

pub(super) fn open_regular(path: &Path) -> ArtifactResult<(File, Stat)> {
    let descriptor = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(open_error)?;
    let state = fstat(&descriptor).map_err(io_error)?;
    require_regular(&state)?;
    Ok((descriptor.into(), state))
}

pub(super) fn require_regular(state: &Stat) -> ArtifactResult<()> {
    if FileType::from_raw_mode(state.st_mode) == FileType::RegularFile {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "verified source must be a regular non-symlink file",
        ))
    }
}

pub(super) fn io_error(error: rustix::io::Errno) -> ArtifactError {
    std::io::Error::from_raw_os_error(error.raw_os_error()).into()
}

pub(super) fn declared_size(state: &Stat) -> ArtifactResult<u64> {
    u64::try_from(state.st_size).map_err(|_| {
        ArtifactError::new(
            ArtifactErrorKind::InvalidContract,
            "source size is negative",
        )
    })
}

pub(super) fn validate_expected(expected: Option<&MediaIdentity>) -> ArtifactResult<()> {
    if expected.is_some_and(|value| {
        value.algorithm != HashAlgorithm::Sha256
            || value.digest.len() != 64
            || !value
                .digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    }) {
        return Err(ArtifactError::new(
            ArtifactErrorKind::InvalidContract,
            "verified source identity must be lowercase SHA-256",
        ));
    }
    Ok(())
}

pub(super) fn same_file_state(left: &Stat, right: &Stat) -> bool {
    left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
        && left.st_size == right.st_size
        && left.st_mtime == right.st_mtime
        && left.st_mtime_nsec == right.st_mtime_nsec
        && left.st_ctime == right.st_ctime
        && left.st_ctime_nsec == right.st_ctime_nsec
}

fn open_error(error: rustix::io::Errno) -> ArtifactError {
    if error == rustix::io::Errno::LOOP {
        ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "verified source must not be a symbolic link",
        )
    } else {
        io_error(error)
    }
}
