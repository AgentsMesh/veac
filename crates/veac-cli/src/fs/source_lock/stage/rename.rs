use std::ffi::OsString;
use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{renameat, statat, AtFlags};
use rustix::io::Errno;

use super::super::path::{self, Identity, Parent};

pub(super) type Renamer = fn(&OwnedFd, &OsString, &OwnedFd, &OsString) -> Result<(), Errno>;

pub(super) fn system(
    source_directory: &OwnedFd,
    source_name: &OsString,
    target_directory: &OwnedFd,
    target_name: &OsString,
) -> Result<(), Errno> {
    renameat(source_directory, source_name, target_directory, target_name)
}

pub(super) fn looks_committed(
    source_directory: &OwnedFd,
    source_name: &OsString,
    target: &Parent,
    expected: Identity,
    label: &Path,
) -> bool {
    let Ok(actual) = path::path_identity(target, label) else {
        return false;
    };
    actual == expected
        && matches!(
            statat(source_directory, source_name, AtFlags::SYMLINK_NOFOLLOW),
            Err(Errno::NOENT)
        )
}
