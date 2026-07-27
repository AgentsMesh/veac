use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

pub(super) fn check(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    if guard() {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::ResourceLimit,
            "artifact cache operation exceeded its caller resource guard",
        ))
    }
}
