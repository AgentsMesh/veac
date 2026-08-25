use std::ffi::OsString;

use rustix::fd::OwnedFd;
use rustix::fs::{renameat_with, statat, AtFlags, RenameFlags};
use rustix::io::Errno;

use super::super::path::{self, Identity, Parent};

pub(super) type Renamer = fn(&OwnedFd, &OsString, &OwnedFd, &OsString) -> Result<(), Errno>;

pub(super) fn system(
    source_directory: &OwnedFd,
    source_name: &OsString,
    target_directory: &OwnedFd,
    target_name: &OsString,
) -> Result<(), Errno> {
    renameat_with(
        source_directory,
        source_name,
        target_directory,
        target_name,
        RenameFlags::EXCHANGE,
    )
}

pub(super) fn replacement_visible(target: &Parent, expected: Identity) -> bool {
    let Ok(actual) = statat(&target.descriptor, &target.name, AtFlags::SYMLINK_NOFOLLOW) else {
        return false;
    };
    path::identity(&actual) == expected
}
