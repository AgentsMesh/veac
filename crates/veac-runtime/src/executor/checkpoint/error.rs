use veac_artifact::{ArtifactError, ArtifactErrorKind};

use crate::RuntimeError;

pub(super) fn artifact(context: &str, error: ArtifactError) -> RuntimeError {
    let message = format!("{context}: {error}");
    if error.kind == ArtifactErrorKind::ResourceLimit {
        RuntimeError::resource_limit(message)
    } else {
        RuntimeError::new(message)
    }
}
