use std::path::Path;

use sha2::{Digest, Sha256};

use super::{io, ArtifactRecord, ArtifactStore};
use crate::{
    artifact_key, ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactResult,
    ContentDigest, DigestAlgorithm,
};

impl ArtifactStore {
    pub fn put(
        &self,
        descriptor: &ArtifactDescriptor,
        payload: &[u8],
    ) -> ArtifactResult<ArtifactRecord> {
        self.put_with(descriptor, payload, |_| {})
    }

    pub fn put_while(
        &self,
        descriptor: &ArtifactDescriptor,
        payload: &[u8],
        guard: impl FnMut() -> bool,
    ) -> ArtifactResult<ArtifactRecord> {
        self.put_while_with(descriptor, payload, |_| {}, guard)
    }

    pub(in crate::cache) fn put_with(
        &self,
        descriptor: &ArtifactDescriptor,
        payload: &[u8],
        before_publish: impl FnOnce(&Path),
    ) -> ArtifactResult<ArtifactRecord> {
        self.put_while_with(descriptor, payload, before_publish, || true)
    }

    pub(in crate::cache) fn put_while_with(
        &self,
        descriptor: &ArtifactDescriptor,
        payload: &[u8],
        before_publish: impl FnOnce(&Path),
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<ArtifactRecord> {
        checked(&mut guard)?;
        if payload.len() as u64 > crate::MAX_IN_MEMORY_ARTIFACT_BYTES {
            return limit("artifact payload exceeds the in-memory cache write limit");
        }
        let key = artifact_key(descriptor)?;
        let record = ArtifactRecord {
            key: key.clone(),
            content: hash_while(payload, &mut guard)?,
            size_bytes: payload.len() as u64,
        };
        if let Some(existing) = self.open_verified_bounded_while(
            &key,
            descriptor,
            crate::MAX_ARTIFACT_PAYLOAD_BYTES,
            &mut guard,
        )? {
            return exact(existing.record(), &record);
        }
        let outcome = io::write_atomic_while_with(
            self.root(),
            &self.directory(&key),
            descriptor,
            &record,
            payload,
            |_| {},
            |_| {},
            before_publish,
            &mut guard,
        )?;
        match outcome {
            io::PublishOutcome::Published => Ok(record),
            io::PublishOutcome::Conflict => self.reuse_winner(descriptor, &record, guard),
        }
    }

    fn reuse_winner(
        &self,
        descriptor: &ArtifactDescriptor,
        expected: &ArtifactRecord,
        guard: impl FnMut() -> bool,
    ) -> ArtifactResult<ArtifactRecord> {
        let existing = self
            .open_verified_bounded_while(
                &expected.key,
                descriptor,
                crate::MAX_ARTIFACT_PAYLOAD_BYTES,
                guard,
            )?
            .ok_or_else(|| {
                ArtifactError::new(
                    ArtifactErrorKind::Io,
                    "concurrent artifact publication vanished before verification",
                )
            })?;
        exact(existing.record(), expected)
    }
}

fn hash_while(payload: &[u8], guard: &mut impl FnMut() -> bool) -> ArtifactResult<ContentDigest> {
    let mut digest = Sha256::new();
    for chunk in payload.chunks(64 * 1024) {
        checked(guard)?;
        digest.update(chunk);
        checked(guard)?;
    }
    Ok(ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: format!("{:x}", digest.finalize()),
    })
}

fn exact(actual: &ArtifactRecord, expected: &ArtifactRecord) -> ArtifactResult<ArtifactRecord> {
    if actual == expected {
        Ok(actual.clone())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::IdentityMismatch,
            "artifact producer returned different content for an existing key",
        ))
    }
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    super::guard::check(guard)
}

fn limit<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        message,
    ))
}

#[cfg(test)]
#[path = "memory/tests.rs"]
mod tests;
