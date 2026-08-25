use std::fs::File;
use std::io::Read;
use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::{fstat, open, openat, statat, AtFlags, FileType, Mode, OFlags, Stat};

use super::{PackageError, PackageErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct FileIdentity {
    device: i128,
    inode: u128,
}

impl FileIdentity {
    fn from_stat(value: &Stat) -> Self {
        Self {
            device: i128::from(value.st_dev),
            inode: u128::from(value.st_ino),
        }
    }
}

#[derive(Debug)]
pub(super) struct BoundDirectory(OwnedFd);

impl BoundDirectory {
    pub(super) fn open(path: &Path) -> Result<Self, PackageError> {
        open(path, directory_flags(), Mode::empty())
            .map(Self)
            .map_err(|error| io(path, error))
    }

    pub(super) fn directory(&self, relative: &str) -> Result<Self, PackageError> {
        super::validation::relative_path(relative)?;
        let mut directory = rustix::io::dup(&self.0)
            .map_err(|error| PackageError::new(PackageErrorKind::Io, error.to_string()))?;
        for name in Path::new(relative)
            .components()
            .map(|part| part.as_os_str())
        {
            directory = openat(&directory, name, directory_flags(), Mode::empty())
                .map_err(|error| unsafe_path(relative, error))?;
        }
        Ok(Self(directory))
    }

    pub(super) fn read(&self, relative: &str, limit: usize) -> Result<Vec<u8>, PackageError> {
        self.read_with_identity(relative, limit)
            .map(|(bytes, _)| bytes)
    }

    pub(super) fn read_with_identity(
        &self,
        relative: &str,
        limit: usize,
    ) -> Result<(Vec<u8>, FileIdentity), PackageError> {
        super::validation::relative_path(relative)?;
        let path = Path::new(relative);
        let mut directory = rustix::io::dup(&self.0)
            .map_err(|error| PackageError::new(PackageErrorKind::Io, error.to_string()))?;
        for name in path
            .parent()
            .into_iter()
            .flat_map(Path::components)
            .map(|part| part.as_os_str())
        {
            directory = openat(&directory, name, directory_flags(), Mode::empty())
                .map_err(|error| unsafe_path(relative, error))?;
        }
        let name = path
            .file_name()
            .expect("validated relative path is non-empty");
        let file = openat(&directory, name, file_flags(), Mode::empty())
            .map_err(|error| unsafe_path(relative, error))?;
        read_stable(file, &directory, Path::new(name), relative, limit)
    }

    pub(super) fn read_text(&self, relative: &str, limit: usize) -> Result<String, PackageError> {
        String::from_utf8(self.read(relative, limit)?).map_err(|error| {
            PackageError::new(
                PackageErrorKind::Json,
                format!("{relative} is not UTF-8: {error}"),
            )
        })
    }
}

fn read_stable(
    file: OwnedFd,
    parent: &OwnedFd,
    name: &Path,
    label: &str,
    limit: usize,
) -> Result<(Vec<u8>, FileIdentity), PackageError> {
    read_stable_with(file, parent, name, label, limit, || {})
}

pub(super) fn read_stable_with(
    file: OwnedFd,
    parent: &OwnedFd,
    name: &Path,
    label: &str,
    limit: usize,
    after_read: impl FnOnce(),
) -> Result<(Vec<u8>, FileIdentity), PackageError> {
    let before = fstat(&file).map_err(|error| io(Path::new(label), error))?;
    if FileType::from_raw_mode(before.st_mode) != FileType::RegularFile
        || before.st_size < 0
        || before.st_size as u64 > limit as u64
    {
        return Err(PackageError::new(
            PackageErrorKind::Io,
            format!("{label} is not a bounded regular file"),
        ));
    }
    let mut file = File::from(file);
    let mut bytes = Vec::with_capacity(before.st_size as usize);
    (&mut file)
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| {
            PackageError::new(
                PackageErrorKind::Io,
                format!("cannot read {label}: {error}"),
            )
        })?;
    after_read();
    let after = fstat(&file).map_err(|error| io(Path::new(label), error))?;
    let current = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|error| io(Path::new(label), error))?;
    if bytes.len() > limit
        || before.st_size as usize != bytes.len()
        || !same(&before, &after)
        || !same(&before, &current)
    {
        return Err(PackageError::new(
            PackageErrorKind::Io,
            format!("{label} changed while it was read"),
        ));
    }
    Ok((bytes, FileIdentity::from_stat(&before)))
}

fn same(left: &Stat, right: &Stat) -> bool {
    FileType::from_raw_mode(right.st_mode) == FileType::RegularFile
        && left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
        && left.st_size == right.st_size
        && left.st_mtime == right.st_mtime
        && left.st_mtime_nsec == right.st_mtime_nsec
        && left.st_ctime == right.st_ctime
        && left.st_ctime_nsec == right.st_ctime_nsec
}

fn directory_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}
fn file_flags() -> OFlags {
    OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC
}
fn unsafe_path(path: &str, error: rustix::io::Errno) -> PackageError {
    PackageError::new(
        PackageErrorKind::RootEscape,
        format!("{path} escapes the package root, uses a symlink, or is unavailable: {error}"),
    )
}
fn io(path: &Path, error: rustix::io::Errno) -> PackageError {
    PackageError::new(
        PackageErrorKind::Io,
        format!("cannot open {}: {error}", path.display()),
    )
}
