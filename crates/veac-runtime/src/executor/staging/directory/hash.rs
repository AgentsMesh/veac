use std::fs::File;
use std::io::Read;
use std::time::Instant;

use rustix::fs::{openat, Mode, OFlags};
use sha2::{Digest, Sha256};
use veac_artifact::{ContentDigest, DigestAlgorithm};

use super::entry::{opened_identity, EntryIdentity};
use super::{failure, Directory};
use crate::executor::deadline;
use crate::RuntimeError;

const HASH_CHUNK_BYTES: usize = 64 * 1024;

impl Directory {
    pub(in crate::executor) fn hash_regular(
        &self,
        name: &str,
        expected: EntryIdentity,
        maximum: u64,
        limit: Instant,
    ) -> Result<(ContentDigest, u64), RuntimeError> {
        deadline::ensure(limit)?;
        let size = expected
            .size_bytes()
            .ok_or_else(|| RuntimeError::new("package member must be a regular file"))?;
        if size > maximum {
            return Err(RuntimeError::resource_limit(
                "delivery package exceeds its output byte budget",
            ));
        }
        self.require(name, expected)?;
        let descriptor = openat(
            &self.descriptor,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| failure(&self.path, "open package member", error))?;
        let mut file = File::from(descriptor);
        if opened_identity(&file)? != expected {
            return Err(RuntimeError::new("opened package member changed identity"));
        }
        let mut digest = Sha256::new();
        let mut counted = 0_u64;
        let mut buffer = [0_u8; HASH_CHUNK_BYTES];
        loop {
            deadline::ensure(limit)?;
            let read = file.read(&mut buffer).map_err(|error| {
                RuntimeError::new(format!("cannot read package member: {error}"))
            })?;
            if read == 0 {
                break;
            }
            counted = counted
                .checked_add(read as u64)
                .ok_or_else(|| RuntimeError::resource_limit("package byte count overflowed"))?;
            if counted > maximum {
                return Err(RuntimeError::resource_limit(
                    "delivery package exceeds its output byte budget",
                ));
            }
            digest.update(&buffer[..read]);
        }
        if counted != size || opened_identity(&file)? != expected {
            return Err(RuntimeError::new(
                "package member changed while it was hashed",
            ));
        }
        self.require(name, expected)?;
        Ok((
            ContentDigest {
                algorithm: DigestAlgorithm::Sha256,
                value: format!("{:x}", digest.finalize()),
            },
            size,
        ))
    }
}
