use std::path::{Path, PathBuf};

use super::{io, ArtifactStore};
use crate::{
    artifact_key, ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactRecord,
    ArtifactResult, ContentDigest,
};

/// An exact store entry verified at open time. Its fields cannot be forged by callers.
///
/// The payload path is not a persistent filesystem capability. Deferred consumers must perform
/// the execution binding's identity revalidation immediately before use.
///
/// ```compile_fail
/// use std::path::PathBuf;
/// use veac_artifact::{ArtifactDescriptor, ArtifactRecord, VerifiedArtifact};
///
/// fn forge(descriptor: ArtifactDescriptor, record: ArtifactRecord) -> VerifiedArtifact {
///     VerifiedArtifact { descriptor, record, payload_path: PathBuf::new() }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedArtifact {
    descriptor: ArtifactDescriptor,
    record: ArtifactRecord,
    payload_path: PathBuf,
}

impl VerifiedArtifact {
    pub fn descriptor(&self) -> &ArtifactDescriptor {
        &self.descriptor
    }

    pub fn record(&self) -> &ArtifactRecord {
        &self.record
    }

    pub fn payload_path(&self) -> &Path {
        &self.payload_path
    }
}

impl ArtifactStore {
    pub fn open(&self, key: &ContentDigest) -> ArtifactResult<Option<VerifiedArtifact>> {
        self.open_while(key, || true)
    }

    pub fn open_while(
        &self,
        key: &ContentDigest,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<Option<VerifiedArtifact>> {
        let Some(inspected) =
            inspect_key_while(self, key, crate::MAX_ARTIFACT_METADATA_BYTES, &mut guard)?
        else {
            return Ok(None);
        };
        finish_inspected_while(inspected, crate::MAX_ARTIFACT_PAYLOAD_BYTES, guard).map(Some)
    }

    pub fn open_verified(
        &self,
        key: &ContentDigest,
        expected: &ArtifactDescriptor,
    ) -> ArtifactResult<Option<VerifiedArtifact>> {
        self.open_verified_bounded(key, expected, crate::MAX_ARTIFACT_PAYLOAD_BYTES)
    }

    pub fn open_verified_bounded(
        &self,
        key: &ContentDigest,
        expected: &ArtifactDescriptor,
        max_payload_bytes: u64,
    ) -> ArtifactResult<Option<VerifiedArtifact>> {
        self.open_verified_bounded_while(key, expected, max_payload_bytes, || true)
    }

    pub fn open_verified_bounded_while(
        &self,
        key: &ContentDigest,
        expected: &ArtifactDescriptor,
        max_payload_bytes: u64,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<Option<VerifiedArtifact>> {
        super::guard::check(&mut guard)?;
        if max_payload_bytes == 0 || max_payload_bytes > crate::MAX_ARTIFACT_PAYLOAD_BYTES {
            return Err(ArtifactError::new(
                ArtifactErrorKind::InvalidContract,
                "artifact payload limit is outside the supported policy",
            ));
        }
        key.validate()?;
        let expected_key = artifact_key(expected)?;
        if expected_key != *key {
            return Err(ArtifactError::new(
                ArtifactErrorKind::IdentityMismatch,
                "expected artifact descriptor does not match the requested key",
            ));
        }
        let Some(inspected) =
            inspect_key_while(self, key, crate::MAX_ARTIFACT_METADATA_BYTES, &mut guard)?
        else {
            return Ok(None);
        };
        if inspected.descriptor != *expected {
            return Err(ArtifactError::new(
                ArtifactErrorKind::IdentityMismatch,
                "stored artifact descriptor does not exactly match the expected descriptor",
            ));
        }
        super::guard::check(&mut guard)?;
        finish_inspected_while(inspected, max_payload_bytes, guard).map(Some)
    }
}

pub(super) fn inspect_key(
    store: &ArtifactStore,
    key: &ContentDigest,
    metadata_budget: u64,
) -> ArtifactResult<Option<io::InspectedArtifact>> {
    inspect_key_while(store, key, metadata_budget, || true)
}

pub(super) fn inspect_key_while(
    store: &ArtifactStore,
    key: &ContentDigest,
    metadata_budget: u64,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<Option<io::InspectedArtifact>> {
    super::guard::check(&mut guard)?;
    key.validate()?;
    let Some(inspected) = io::inspect_bounded_while(
        store.root(),
        &store.directory(key),
        metadata_budget,
        &mut guard,
    )?
    else {
        return Ok(None);
    };
    super::guard::check(&mut guard)?;
    if artifact_key(&inspected.descriptor).map_err(cache_metadata)? != *key
        || inspected.record.key != *key
    {
        return Err(ArtifactError::new(
            ArtifactErrorKind::CorruptCache,
            "cached artifact key does not match its metadata",
        ));
    }
    Ok(Some(inspected))
}

pub(super) fn finish_inspected(
    inspected: io::InspectedArtifact,
    payload_budget: u64,
) -> ArtifactResult<VerifiedArtifact> {
    finish_inspected_while(inspected, payload_budget, || true)
}

pub(super) fn finish_inspected_while(
    mut inspected: io::InspectedArtifact,
    payload_budget: u64,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<VerifiedArtifact> {
    super::guard::check(&mut guard)?;
    let payload_path =
        io::verify_payload_file_bounded_while(&mut inspected, payload_budget, &mut guard)?;
    super::guard::check(&mut guard)?;
    Ok(VerifiedArtifact {
        descriptor: inspected.descriptor,
        record: inspected.record,
        payload_path,
    })
}

fn cache_metadata(error: ArtifactError) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::CorruptCache,
        "cached artifact descriptor failed identity verification",
        error,
    )
}

#[cfg(test)]
#[path = "verified/tests.rs"]
mod tests;
