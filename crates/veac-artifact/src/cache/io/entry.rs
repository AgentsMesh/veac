use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use rustix::fs::{fstat, openat, unlinkat, AtFlags, FileType, Mode, OFlags};
use sha2::{Digest, Sha256};

use super::authority::verify_identity;
use super::common::{corrupt, corrupt_io, io_error, resource_limit};
use crate::{ArtifactResult, ContentDigest, DigestAlgorithm};

const READ_FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);

#[derive(Debug)]
pub(super) struct BoundFile {
    name: OsString,
    file: File,
}

impl BoundFile {
    pub(super) fn open(directory: &File, name: &OsStr) -> ArtifactResult<Self> {
        let file = openat(directory, name, READ_FLAGS, Mode::empty())
            .map(File::from)
            .map_err(|error| corrupt_io("open cached artifact entry", error))?;
        require_regular(&file)?;
        Ok(Self {
            name: name.to_owned(),
            file,
        })
    }

    pub(super) fn create(directory: &File, name: &OsStr) -> ArtifactResult<Self> {
        let flags = READ_FLAGS | OFlags::RDWR | OFlags::CREATE | OFlags::EXCL;
        let file = openat(directory, name, flags, Mode::RUSR | Mode::WUSR)
            .map(File::from)
            .map_err(|error| io_error("create staged cache entry", error))?;
        Ok(Self {
            name: name.to_owned(),
            file,
        })
    }

    pub(super) fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    pub(super) fn size(&self) -> ArtifactResult<u64> {
        let stat = fstat(&self.file).map_err(|error| io_error("inspect cache entry", error))?;
        Ok(stat.st_size as u64)
    }

    pub(super) fn read_bounded_while(
        &mut self,
        limit: u64,
        message: &str,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<Vec<u8>> {
        crate::cache::guard::check(&mut guard)?;
        let size = self.size()?;
        if size > limit {
            return resource_limit(message);
        }
        self.file.seek(SeekFrom::Start(0))?;
        let capacity =
            usize::try_from(size).map_err(|_| resource_limit::<()>(message).unwrap_err())?;
        let mut bytes = Vec::with_capacity(capacity);
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            crate::cache::guard::check(&mut guard)?;
            let count = self.file.read(&mut buffer)?;
            crate::cache::guard::check(&mut guard)?;
            if count == 0 {
                break;
            }
            if bytes.len().saturating_add(count) as u64 > limit {
                return resource_limit(message);
            }
            bytes.extend_from_slice(&buffer[..count]);
        }
        Ok(bytes)
    }

    pub(super) fn rewind(&mut self) -> ArtifactResult<()> {
        self.file.seek(SeekFrom::Start(0))?;
        Ok(())
    }

    pub(super) fn fingerprint_while(
        &mut self,
        limit: u64,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<(ContentDigest, u64)> {
        crate::cache::guard::check(&mut guard)?;
        self.rewind()?;
        let mut digest = Sha256::new();
        let mut size = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            crate::cache::guard::check(&mut guard)?;
            let count = self.file.read(&mut buffer)?;
            crate::cache::guard::check(&mut guard)?;
            if count == 0 {
                break;
            }
            size = size.checked_add(count as u64).ok_or_else(|| {
                resource_limit::<()>("artifact payload size overflow").unwrap_err()
            })?;
            if size > limit {
                return resource_limit("artifact payload exceeds the cache limit");
            }
            digest.update(&buffer[..count]);
        }
        Ok((
            ContentDigest {
                algorithm: DigestAlgorithm::Sha256,
                value: format!("{:x}", digest.finalize()),
            },
            size,
        ))
    }

    pub(super) fn sync(&self) -> ArtifactResult<()> {
        self.file.sync_all()?;
        Ok(())
    }

    pub(super) fn verify_while(
        &self,
        directory: &File,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<()> {
        crate::cache::guard::check(&mut guard)?;
        let current = openat(directory, &self.name, READ_FLAGS, Mode::empty())
            .map(File::from)
            .map_err(|error| corrupt_io("reopen cached artifact entry", error))?;
        crate::cache::guard::check(&mut guard)?;
        require_regular(&current)?;
        crate::cache::guard::check(&mut guard)?;
        verify_identity(&self.file, &current, FileType::RegularFile)?;
        crate::cache::guard::check(&mut guard)
    }

    pub(super) fn unlink(&self, directory: &File) -> ArtifactResult<()> {
        self.verify_while(directory, || true)?;
        unlinkat(directory, &self.name, AtFlags::empty())
            .map_err(|error| io_error("remove cached artifact entry", error))
    }
}

fn require_regular(file: &File) -> ArtifactResult<()> {
    let stat = fstat(file).map_err(|error| io_error("inspect cached artifact entry", error))?;
    if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile || stat.st_nlink != 1 {
        return corrupt("cached artifact entry is not an exclusively linked regular file");
    }
    Ok(())
}

#[cfg(test)]
#[path = "entry/tests.rs"]
mod tests;
