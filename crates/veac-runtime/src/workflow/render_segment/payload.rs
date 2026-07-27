use std::path::Path;
use std::time::Instant;

use veac_artifact::{
    verify_source_prefix_bounded_while, ArtifactErrorKind, ArtifactRecord, VerifiedSourcePrefix,
    MAX_VERIFIED_SOURCE_PREFIX_BYTES,
};
use veac_ir::MediaIdentity;

use super::{identity_error, WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) fn verify(
    path: &Path,
    identity: &MediaIdentity,
    record: &ArtifactRecord,
    max_bytes: u64,
    deadline: Instant,
) -> WorkflowResult<VerifiedSourcePrefix> {
    let source = verify_source_prefix_bounded_while(
        path,
        Some(identity),
        max_bytes,
        MAX_VERIFIED_SOURCE_PREFIX_BYTES,
        || Instant::now() < deadline,
    )
    .map_err(|error| {
        let kind = match error.kind {
            ArtifactErrorKind::IdentityMismatch => WorkflowErrorKind::SourceIdentityMismatch,
            ArtifactErrorKind::ResourceLimit => WorkflowErrorKind::ResourceLimit,
            _ => WorkflowErrorKind::Artifact,
        };
        WorkflowError::with_source(kind, "render-segment payload verification failed", error)
    })?;
    if source.verified.size_bytes != record.size_bytes {
        return Err(identity_error(
            "render-segment payload size differs from its record",
        ));
    }
    Ok(source)
}
