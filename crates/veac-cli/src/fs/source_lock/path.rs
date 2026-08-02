use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path};

use rustix::fd::OwnedFd;
use rustix::fs::{fstat, openat, statat, AtFlags, FileType, Mode, OFlags};

use crate::error::{CliError, CliResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Identity {
    device: i128,
    inode: u128,
}

pub(super) struct Parent {
    pub(super) descriptor: OwnedFd,
    pub(super) identity: Identity,
    pub(super) name: OsString,
}

pub(super) struct Target {
    pub(super) bytes: Vec<u8>,
    pub(super) identity: Identity,
    pub(super) mode: Mode,
}

pub(super) fn resolve(root: &OwnedFd, module: &str, label: &Path) -> CliResult<Parent> {
    veac_lang::source_edit::validate_module_path(module)
        .map_err(|error| CliError::new("SOURCE_EDIT_MODULE", error.to_string()))?;
    let mut parts = Path::new(module).components().peekable();
    let mut directory = cli_try!(rustix::io::dup(root), |error| changed(label, error));
    while let Some(component) = parts.next() {
        let Component::Normal(name) = component else {
            return Err(changed_message(label, "module path is not confined"));
        };
        if parts.peek().is_none() {
            let stat = cli_try!(fstat(&directory), |error| changed(label, error));
            return Ok(Parent {
                identity: identity(&stat),
                descriptor: directory,
                name: name.to_owned(),
            });
        }
        directory = openat(
            &directory,
            Path::new(name),
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| changed(label, error))?;
    }
    Err(changed_message(label, "module path is empty"))
}

pub(super) fn require_parent(
    root: &OwnedFd,
    module: &str,
    expected: Identity,
    label: &Path,
) -> CliResult {
    let current = resolve(root, module, label)?;
    if current.identity == expected {
        Ok(())
    } else {
        Err(changed_message(label, "module parent changed"))
    }
}

pub(super) fn read_target(parent: &Parent, label: &Path, maximum: usize) -> CliResult<Target> {
    read_target_with(parent, label, maximum, || {})
}

pub(super) fn read_target_with(
    parent: &Parent,
    label: &Path,
    maximum: usize,
    after_read: impl FnOnce(),
) -> CliResult<Target> {
    let descriptor = openat(
        &parent.descriptor,
        &parent.name,
        OFlags::RDONLY | OFlags::NONBLOCK | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| changed(label, error))?;
    let before = cli_try!(fstat(&descriptor), |error| changed(label, error));
    if FileType::from_raw_mode(before.st_mode) != FileType::RegularFile
        || before.st_size < 0
        || before.st_size as u64 > maximum as u64
    {
        return Err(changed_message(
            label,
            "module is not the expected regular file",
        ));
    }
    let mut reader = File::from(descriptor).take(maximum.saturating_add(1) as u64);
    let mut bytes = Vec::with_capacity(before.st_size as usize);
    cli_try!(reader.read_to_end(&mut bytes), |error| changed_message(
        label,
        &format!("cannot read module: {error}")
    ));
    after_read();
    let file = reader.into_inner();
    let after = cli_try!(fstat(&file), |error| changed(label, error));
    let current = statat(&parent.descriptor, &parent.name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| changed(label, error))?;
    let expected = identity(&before);
    if bytes.len() > maximum
        || before.st_size as usize != bytes.len()
        || !same_file_state(&before, &after)
        || !same_file_state(&before, &current)
    {
        return Err(changed_message(label, "module changed while it was read"));
    }
    Ok(Target {
        bytes,
        identity: expected,
        mode: Mode::from_raw_mode(before.st_mode),
    })
}

fn same_file_state(left: &rustix::fs::Stat, right: &rustix::fs::Stat) -> bool {
    left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
        && left.st_mode == right.st_mode
        && left.st_size == right.st_size
        && left.st_mtime == right.st_mtime
        && left.st_mtime_nsec == right.st_mtime_nsec
        && left.st_ctime == right.st_ctime
        && left.st_ctime_nsec == right.st_ctime_nsec
}

pub(super) fn path_identity(parent: &Parent, label: &Path) -> CliResult<Identity> {
    statat(&parent.descriptor, &parent.name, AtFlags::SYMLINK_NOFOLLOW)
        .map(|value| identity(&value))
        .map_err(|error| changed(label, error))
}

pub(super) fn identity(stat: &rustix::fs::Stat) -> Identity {
    Identity {
        device: i128::from(stat.st_dev),
        inode: u128::from(stat.st_ino),
    }
}

fn changed(label: &Path, error: rustix::io::Errno) -> CliError {
    changed_message(label, &error.to_string())
}

fn changed_message(label: &Path, message: &str) -> CliError {
    CliError::new(
        "SOURCE_CHANGED",
        format!(
            "source module {} changed during the edit: {message}",
            label.display()
        ),
    )
}
