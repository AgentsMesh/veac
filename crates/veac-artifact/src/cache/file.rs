use std::path::Path;

use super::{ArtifactRecord, ArtifactStore};
use crate::{
    artifact_key, ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactResult,
    ContentDigest,
};

mod store;

impl ArtifactStore {
    pub fn put_file(
        &self,
        descriptor: &ArtifactDescriptor,
        source: &Path,
    ) -> ArtifactResult<ArtifactRecord> {
        store::put_file_inner_while(self, descriptor, source, None, || true)
            .map(|stored| stored.record)
    }

    pub fn put_file_expected(
        &self,
        descriptor: &ArtifactDescriptor,
        source: &Path,
        expected_content: &ContentDigest,
        expected_size: u64,
    ) -> ArtifactResult<ArtifactRecord> {
        self.put_file_expected_while(descriptor, source, expected_content, expected_size, || true)
    }

    pub fn put_file_expected_while(
        &self,
        descriptor: &ArtifactDescriptor,
        source: &Path,
        expected_content: &ContentDigest,
        expected_size: u64,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<ArtifactRecord> {
        super::guard::check(&mut guard)?;
        expected_content.validate()?;
        if expected_size > crate::MAX_ARTIFACT_PAYLOAD_BYTES {
            return limit("expected artifact payload exceeds the cache limit");
        }
        store::put_file_inner_while(
            self,
            descriptor,
            source,
            Some((expected_content, expected_size)),
            guard,
        )
        .map(|stored| stored.record)
    }

    pub fn put_verified_file(
        &self,
        descriptor: &ArtifactDescriptor,
        declared: &ArtifactRecord,
        path: &Path,
    ) -> ArtifactResult<ArtifactRecord> {
        self.put_verified_file_while(descriptor, declared, path, || true)
    }

    pub fn put_verified_file_while(
        &self,
        descriptor: &ArtifactDescriptor,
        declared: &ArtifactRecord,
        path: &Path,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<ArtifactRecord> {
        super::guard::check(&mut guard)?;
        if declared.key != artifact_key(descriptor)? {
            return Err(ArtifactError::new(
                ArtifactErrorKind::IdentityMismatch,
                "artifact declaration key does not match its descriptor",
            ));
        }
        declared.content.validate()?;
        if declared.size_bytes > crate::MAX_ARTIFACT_PAYLOAD_BYTES {
            return limit("declared artifact payload exceeds the cache limit");
        }
        let stored = store::put_file_inner_while(
            self,
            descriptor,
            path,
            Some((&declared.content, declared.size_bytes)),
            &mut guard,
        )?;
        super::guard::check(&mut guard).map_err(|error| stored.map_error(error))?;
        if stored.record != *declared {
            return Err(stored.map_error(ArtifactError::new(
                ArtifactErrorKind::IdentityMismatch,
                "stored artifact record differs from the declaration",
            )));
        }
        Ok(stored.record)
    }
}

fn limit<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        message,
    ))
}
