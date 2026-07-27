use crate::ArtifactResult;

pub(super) fn bounded_json(value: &impl serde::Serialize, limit: u64) -> ArtifactResult<()> {
    crate::json::canonical_bounded(value, limit, "artifact metadata cannot be canonicalized")
        .map(drop)
}
