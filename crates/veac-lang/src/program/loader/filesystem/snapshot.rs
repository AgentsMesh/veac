use std::fs::File;
use std::io::Read;
use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{fstat, statat, AtFlags, FileType, Stat};

use super::identity::FileIdentity;
use super::OpenedSource;
use crate::program::limits::MAX_SOURCE_BYTES;

pub(super) fn read_regular(
    file: OwnedFd,
    parent: &OwnedFd,
    name: &Path,
    label: &Path,
) -> Result<OpenedSource, String> {
    read_regular_with(file, parent, name, label, || {})
}

fn read_regular_with(
    file: OwnedFd,
    parent: &OwnedFd,
    name: &Path,
    label: &Path,
    after_read: impl FnOnce(),
) -> Result<OpenedSource, String> {
    let before = fstat(&file).map_err(|error| inspect_error(label, error))?;
    if !is_regular(&before) {
        return Err(format!("source {} is not a regular file", label.display()));
    }
    if before.st_size < 0 || before.st_size as u64 > MAX_SOURCE_BYTES as u64 {
        return Err(format!("source {} exceeds 16 MiB", label.display()));
    }
    let mut file = File::from(file);
    let mut bytes = Vec::with_capacity(before.st_size as usize);
    (&mut file)
        .take((MAX_SOURCE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read source {}: {error}", label.display()))?;
    after_read();
    let after = fstat(&file).map_err(|error| changed_error(label, error))?;
    let current = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| changed_error(label, error))?;
    if bytes.len() > MAX_SOURCE_BYTES
        || usize::try_from(before.st_size).ok() != Some(bytes.len())
        || !same_state(&before, &after)
        || !same_state(&before, &current)
    {
        return Err(format!(
            "source {} changed while it was read",
            label.display()
        ));
    }
    let source = String::from_utf8(bytes)
        .map_err(|error| format!("source {} is not UTF-8: {error}", label.display()))?;
    Ok(OpenedSource {
        source,
        identity: FileIdentity::from_stat(&before),
    })
}

fn is_regular(value: &Stat) -> bool {
    FileType::from_raw_mode(value.st_mode) == FileType::RegularFile
}

fn same_state(left: &Stat, right: &Stat) -> bool {
    is_regular(right)
        && left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
        && left.st_size == right.st_size
        && left.st_mtime == right.st_mtime
        && left.st_mtime_nsec == right.st_mtime_nsec
        && left.st_ctime == right.st_ctime
        && left.st_ctime_nsec == right.st_ctime_nsec
}

fn inspect_error(label: &Path, error: rustix::io::Errno) -> String {
    format!("cannot inspect source {}: {error}", label.display())
}

fn changed_error(label: &Path, error: rustix::io::Errno) -> String {
    format!(
        "source {} changed while it was read: {error}",
        label.display()
    )
}

#[cfg(test)]
#[path = "snapshot/tests.rs"]
mod tests;
