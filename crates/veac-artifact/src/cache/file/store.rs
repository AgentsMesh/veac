use std::path::Path;

use super::*;
use crate::cache::io;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Publication {
    Published,
    Reused,
}

pub(super) struct StoredFile {
    pub record: ArtifactRecord,
    publication: Publication,
}

impl StoredFile {
    pub(super) fn map_error(&self, error: ArtifactError) -> ArtifactError {
        if self.publication == Publication::Published {
            error.committed()
        } else {
            error
        }
    }
}

pub(super) fn put_file_inner_while(
    store: &ArtifactStore,
    descriptor: &ArtifactDescriptor,
    source: &Path,
    expected: Option<(&ContentDigest, u64)>,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<StoredFile> {
    super::super::guard::check(&mut guard)?;
    let key = artifact_key(descriptor)?;
    super::super::guard::check(&mut guard)?;
    let (content, size_bytes) = io::fingerprint_source_while(source, &mut guard)?;
    if expected.is_some_and(|(digest, size)| digest != &content || size != size_bytes) {
        return mismatch("artifact source differs from the runtime-verified output");
    }
    let record = ArtifactRecord {
        key: key.clone(),
        content,
        size_bytes,
    };
    let payload_limit = crate::MAX_ARTIFACT_PAYLOAD_BYTES;
    if let Some(existing) =
        store.open_verified_bounded_while(&key, descriptor, payload_limit, &mut guard)?
    {
        if existing.record() == &record {
            return Ok(stored(record, Publication::Reused));
        }
        return mismatch("artifact producer returned different content for an existing key");
    }
    let outcome = io::write_file_atomic_while(
        store.root(),
        &store.directory(&key),
        descriptor,
        &record,
        source,
        &mut guard,
    )?;
    let publication = match outcome {
        io::PublishOutcome::Published => Publication::Published,
        io::PublishOutcome::Conflict => Publication::Reused,
    };
    let result = StoredFile {
        record: record.clone(),
        publication,
    };
    let opened = store
        .open_verified_bounded_while(&key, descriptor, payload_limit, &mut guard)
        .map_err(|error| result.map_error(error))?
        .ok_or_else(|| {
            result.map_error(ArtifactError::new(
                ArtifactErrorKind::Io,
                "stored artifact vanished",
            ))
        })?;
    if opened.record() != &record {
        return Err(result.map_error(ArtifactError::new(
            ArtifactErrorKind::IdentityMismatch,
            "stored artifact record differs from the streamed source",
        )));
    }
    Ok(result)
}

fn stored(record: ArtifactRecord, publication: Publication) -> StoredFile {
    StoredFile {
        record,
        publication,
    }
}

fn mismatch<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::IdentityMismatch,
        message,
    ))
}
