use veac_artifact::{ArtifactStore, ContentDigest};

use crate::RuntimeError;

pub(in crate::executor) fn after_failure(
    store: &ArtifactStore,
    key: &ContentDigest,
    error: RuntimeError,
) -> RuntimeError {
    // Consistency cleanup is rollback work and must still run after the task deadline expires.
    match store.remove(key) {
        Ok(_) => error,
        Err(cleanup) => RuntimeError {
            kind: error.kind,
            message: format!("{error}; cannot discard fresh render checkpoint: {cleanup}"),
        },
    }
}
