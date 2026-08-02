use std::path::{Path, PathBuf};

use super::authority::target_parent;
use super::common::{corrupt, serialization_error};
use super::directory::BoundDirectory;
use super::entry::BoundFile;
use super::lock::DirectoryLock;
use super::state;
use crate::{
    canonical_descriptor_bytes, ArtifactDescriptor, ArtifactError, ArtifactErrorKind,
    ArtifactRecord, ArtifactResult, MAX_ARTIFACT_METADATA_BYTES, MAX_ARTIFACT_PAYLOAD_BYTES,
    MAX_IN_MEMORY_ARTIFACT_BYTES,
};

mod guarded;
mod payload;

pub(in crate::cache) struct InspectedArtifact {
    pub descriptor: ArtifactDescriptor,
    pub record: ArtifactRecord,
    payload: BoundFile,
    descriptor_file: BoundFile,
    record_file: BoundFile,
    marker: BoundFile,
    directory: BoundDirectory,
    metadata_bytes: u64,
    _prefix_lock: DirectoryLock,
}

impl InspectedArtifact {
    pub(super) fn read_payload_while(
        &mut self,
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<Vec<u8>> {
        crate::cache::guard::check(&mut guard)?;
        if self.record.size_bytes > MAX_IN_MEMORY_ARTIFACT_BYTES {
            return super::common::resource_limit(
                "artifact payload exceeds the in-memory cache read limit",
            );
        }
        let payload = self.payload.read_bounded_while(
            MAX_IN_MEMORY_ARTIFACT_BYTES,
            "artifact payload exceeds the in-memory cache read limit",
            &mut guard,
        )?;
        payload::verify_bytes_while(&self.record, &payload, &mut guard)?;
        guarded::verify_entries(self, &mut guard)?;
        Ok(payload)
    }

    pub(in crate::cache) fn metadata_bytes(&self) -> u64 {
        self.metadata_bytes
    }
}

pub(super) fn inspect_bounded_while_with(
    root: &Path,
    directory: &Path,
    metadata_budget: u64,
    after_entry_open: impl FnOnce(&Path),
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<Option<InspectedArtifact>> {
    guarded::inspect_bounded_with(
        root,
        directory,
        metadata_budget,
        after_entry_open,
        &mut guard,
    )
}

pub(in crate::cache) fn inspect_bounded_while(
    root: &Path,
    directory: &Path,
    metadata_budget: u64,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<Option<InspectedArtifact>> {
    inspect_bounded_while_with(root, directory, metadata_budget, |_| {}, &mut guard)
}

pub(in crate::cache) fn verify_payload_file_bounded_while(
    inspected: &mut InspectedArtifact,
    payload_budget: u64,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<PathBuf> {
    crate::cache::guard::check(&mut guard)?;
    if inspected.record.size_bytes > MAX_ARTIFACT_PAYLOAD_BYTES
        || inspected.record.size_bytes > payload_budget
    {
        return super::common::resource_limit("cached artifact payload exceeds the disk limit");
    }
    let (digest, size) = inspected
        .payload
        .fingerprint_while(payload_budget, &mut guard)?;
    if size != inspected.record.size_bytes || digest != inspected.record.content {
        return corrupt("cached artifact payload failed identity verification");
    }
    guarded::verify_entries(inspected, &mut guard)?;
    Ok(inspected.directory.path().join(state::PAYLOAD))
}

fn validate_record(record: &ArtifactRecord, bytes: &[u8]) -> ArtifactResult<()> {
    record.key.validate().map_err(metadata_error)?;
    record.content.validate().map_err(metadata_error)?;
    if record.size_bytes > MAX_ARTIFACT_PAYLOAD_BYTES {
        return corrupt("cached artifact record exceeds the payload limit");
    }
    let canonical = serde_json_canonicalizer::to_vec(record).map_err(serialization_error)?;
    if canonical != bytes {
        return corrupt("cached artifact record is not canonical JSON");
    }
    Ok(())
}

fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> ArtifactResult<T> {
    serde_json::from_slice(bytes).map_err(serialization_error)
}

fn metadata_error(error: ArtifactError) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::CorruptCache,
        "cached artifact metadata failed validation",
        error,
    )
}
