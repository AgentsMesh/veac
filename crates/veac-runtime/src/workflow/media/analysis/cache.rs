use veac_artifact::{
    canonical_artifact_json_bounded, read_verified_source_bounded_while, validate_artifact_json,
    ContentDigest, VerifiedArtifact,
};
use veac_ir::{HashAlgorithm, MediaIdentity};

use super::super::{artifact_error, contract, WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) fn validate(
    artifact: &VerifiedArtifact,
    limit: u64,
    guard: &mut impl FnMut() -> bool,
) -> WorkflowResult<()> {
    if artifact.record().size_bytes > limit {
        return Err(WorkflowError::new(
            WorkflowErrorKind::ResourceLimit,
            "cached analysis artifact exceeds the active payload budget",
        ));
    }
    let identity = identity(&artifact.record().content);
    let bytes = read_verified_source_bounded_while(
        artifact.payload_path(),
        Some(&identity),
        limit,
        &mut *guard,
    )
    .map_err(artifact_error)?;
    super::active(guard)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|error| {
        WorkflowError::with_source(
            WorkflowErrorKind::Artifact,
            "cached analysis artifact is not valid JSON",
            error,
        )
    })?;
    super::active(guard)?;
    if !value.is_object() || validate_artifact_json(&value).is_err() {
        return Err(WorkflowError::new(
            WorkflowErrorKind::Artifact,
            "cached analysis artifact violates its JSON contract",
        ));
    }
    super::active(guard)?;
    let canonical = contract(canonical_artifact_json_bounded(&value, limit))?;
    super::active(guard)?;
    if canonical != bytes {
        return Err(WorkflowError::new(
            WorkflowErrorKind::Artifact,
            "cached analysis artifact is not canonical JSON",
        ));
    }
    Ok(())
}

fn identity(digest: &ContentDigest) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: digest.value.clone(),
    }
}
