use std::time::Instant;

use veac_artifact::{
    ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactRecord, ArtifactStore,
    ContentDigest, MediaArtifactLimits,
};
use veac_ir::{HashAlgorithm, MediaIdentity};

use super::{postflight, WorkflowError, WorkflowErrorKind, WorkflowResult};
use crate::asset::SystemFfprobe;

pub(super) fn load(
    store: &ArtifactStore,
    key: &ContentDigest,
    descriptor: &ArtifactDescriptor,
    ffprobe: &SystemFfprobe,
    spec: &veac_artifact::MediaArtifactSpec,
    limits: MediaArtifactLimits,
    deadline: Instant,
) -> WorkflowResult<Option<ArtifactRecord>> {
    let max_bytes = limits.max_payload_bytes;
    let Some(artifact) = store
        .open_verified_bounded_while(key, descriptor, max_bytes, || Instant::now() < deadline)
        .map_err(cache_error)?
    else {
        return Ok(None);
    };
    if artifact.record().size_bytes > max_bytes {
        return limit("cached artifact exceeds the active payload budget");
    }
    verify(&artifact, max_bytes, deadline)?;
    postflight::validate(ffprobe, artifact.payload_path(), spec, deadline)?;
    verify(&artifact, max_bytes, deadline)?;
    Ok(Some(artifact.record().clone()))
}

fn verify(
    artifact: &veac_artifact::VerifiedArtifact,
    max_bytes: u64,
    deadline: Instant,
) -> WorkflowResult<()> {
    let identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: artifact.record().content.value.clone(),
    };
    veac_artifact::verify_source_bounded_while(
        artifact.payload_path(),
        Some(&identity),
        max_bytes,
        || Instant::now() < deadline,
    )
    .map(|_| ())
    .map_err(cache_error)
}

fn cache_error(error: ArtifactError) -> WorkflowError {
    let kind = if error.kind == ArtifactErrorKind::ResourceLimit {
        WorkflowErrorKind::ResourceLimit
    } else {
        WorkflowErrorKind::Artifact
    };
    WorkflowError::with_source(kind, "cached artifact verification failed", error)
}

fn limit<T>(message: &str) -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::ResourceLimit,
        message,
    ))
}

#[cfg(test)]
#[path = "cache/tests.rs"]
mod tests;
