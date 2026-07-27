use std::path::{Path, PathBuf};

use crate::{
    ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactRecord, ArtifactResult,
    ContentDigest,
};

use super::PublishOutcome;

pub(in crate::cache) struct InspectedArtifact {
    pub descriptor: ArtifactDescriptor,
    pub record: ArtifactRecord,
}

impl InspectedArtifact {
    pub(in crate::cache) fn metadata_bytes(&self) -> u64 {
        0
    }
}

pub(in crate::cache) fn read_while(
    _: &Path,
    _: &Path,
    _: impl FnMut() -> bool,
) -> ArtifactResult<Option<(ArtifactDescriptor, ArtifactRecord, Vec<u8>)>> {
    Err(unsupported())
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cache) fn write_atomic_while_with(
    _: &Path,
    _: &Path,
    _: &ArtifactDescriptor,
    _: &ArtifactRecord,
    _: &[u8],
    _: impl FnOnce(&Path),
    _: impl FnOnce(&Path),
    _: impl FnOnce(&Path),
    _: impl FnMut() -> bool,
) -> ArtifactResult<PublishOutcome> {
    Err(unsupported())
}

pub(in crate::cache) fn write_file_atomic_while(
    _: &Path,
    _: &Path,
    _: &ArtifactDescriptor,
    _: &ArtifactRecord,
    _: &Path,
    _: impl FnMut() -> bool,
) -> ArtifactResult<PublishOutcome> {
    Err(unsupported())
}

pub(in crate::cache) fn inspect_bounded_while(
    _: &Path,
    _: &Path,
    _: u64,
    _: impl FnMut() -> bool,
) -> ArtifactResult<Option<InspectedArtifact>> {
    Err(unsupported())
}

pub(in crate::cache) fn verify_payload_file_bounded_while(
    _: &mut InspectedArtifact,
    _: u64,
    _: impl FnMut() -> bool,
) -> ArtifactResult<PathBuf> {
    Err(unsupported())
}

pub(in crate::cache) fn remove_while(
    _: &Path,
    _: &Path,
    _: impl FnMut() -> bool,
) -> ArtifactResult<bool> {
    Err(unsupported())
}

pub(in crate::cache) fn catalog_keys(_: &Path) -> ArtifactResult<Vec<ContentDigest>> {
    Err(unsupported())
}

pub(in crate::cache) fn fingerprint_source_while(
    _: &Path,
    _: impl FnMut() -> bool,
) -> ArtifactResult<(ContentDigest, u64)> {
    Err(unsupported())
}

fn unsupported() -> ArtifactError {
    ArtifactError::new(
        ArtifactErrorKind::UnsafePath,
        "descriptor-relative cache access is unsupported on this platform",
    )
}
