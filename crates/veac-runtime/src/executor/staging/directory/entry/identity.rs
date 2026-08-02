use rustix::fs::{fstat, FileType, Stat};

use super::model::EntryIdentity;
use crate::RuntimeError;

pub(super) fn identity(
    parent: &std::path::Path,
    name: &str,
    metadata: Stat,
) -> Result<EntryIdentity, RuntimeError> {
    let device = identity_number(metadata.st_dev, "device number")?;
    let inode = identity_number(metadata.st_ino, "inode number")?;
    match FileType::from_raw_mode(metadata.st_mode) {
        FileType::RegularFile if metadata.st_nlink == 1 => Ok(EntryIdentity::Regular {
            device,
            inode,
            size_bytes: nonnegative_size(metadata.st_size)?,
        }),
        FileType::Directory => Ok(EntryIdentity::Directory { device, inode }),
        _ => Err(RuntimeError::new(format!(
            "descriptor-relative path {} must be a directory or exclusively linked regular file",
            parent.join(name).display()
        ))),
    }
}

pub(in crate::executor::staging::directory) fn opened_identity(
    file: &std::fs::File,
) -> Result<EntryIdentity, RuntimeError> {
    let metadata = fstat(file).map_err(|error| {
        RuntimeError::new(format!("cannot inspect opened transaction file: {error}"))
    })?;
    match from_stat(metadata)? {
        value @ EntryIdentity::Regular { .. } if metadata.st_nlink == 1 => Ok(value),
        _ => Err(RuntimeError::new(
            "opened transaction file must be an exclusively linked regular file",
        )),
    }
}

pub(in crate::executor::staging::directory) fn from_stat(
    metadata: Stat,
) -> Result<EntryIdentity, RuntimeError> {
    let device = identity_number(metadata.st_dev, "device number")?;
    let inode = identity_number(metadata.st_ino, "inode number")?;
    match FileType::from_raw_mode(metadata.st_mode) {
        FileType::RegularFile => Ok(EntryIdentity::Regular {
            device,
            inode,
            size_bytes: nonnegative_size(metadata.st_size)?,
        }),
        FileType::Directory => Ok(EntryIdentity::Directory { device, inode }),
        _ => Err(RuntimeError::new(
            "opened transaction node has an unsupported type",
        )),
    }
}

pub(super) fn identity_number<T: TryInto<u64>>(value: T, name: &str) -> Result<u64, RuntimeError> {
    value
        .try_into()
        .map_err(|_| RuntimeError::new(format!("descriptor-relative {name} is out of range")))
}

pub(super) fn nonnegative_size<T: TryInto<u64>>(value: T) -> Result<u64, RuntimeError> {
    value
        .try_into()
        .map_err(|_| RuntimeError::new("descriptor-relative file size is negative"))
}
