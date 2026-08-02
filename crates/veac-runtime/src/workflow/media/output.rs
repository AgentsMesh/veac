use std::path::Path;
use std::time::Instant;

use veac_artifact::{
    verify_source_bounded_while, ArtifactDescriptor, ArtifactRecord, ArtifactStore, ContentDigest,
    DigestAlgorithm,
};
use veac_ir::MediaIdentity;

use super::{artifact_error, WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) struct OutputProof {
    identity: MediaIdentity,
    size_bytes: u64,
}

impl OutputProof {
    pub(super) fn capture(path: &Path, max_bytes: u64, deadline: Instant) -> WorkflowResult<Self> {
        let verified =
            verify_source_bounded_while(path, None, max_bytes, || Instant::now() < deadline)
                .map_err(artifact_error)?;
        Ok(Self {
            identity: verified.identity,
            size_bytes: verified.size_bytes,
        })
    }

    pub(super) fn reverify(
        &self,
        path: &Path,
        max_bytes: u64,
        deadline: Instant,
    ) -> WorkflowResult<()> {
        let verified = verify_source_bounded_while(path, Some(&self.identity), max_bytes, || {
            Instant::now() < deadline
        })
        .map_err(artifact_error)?;
        if verified.size_bytes != self.size_bytes {
            return Err(WorkflowError::new(
                WorkflowErrorKind::Artifact,
                "rendered artifact size changed after postflight",
            ));
        }
        Ok(())
    }

    pub(super) fn store(
        &self,
        store: &ArtifactStore,
        descriptor: &ArtifactDescriptor,
        path: &Path,
        deadline: Instant,
    ) -> WorkflowResult<ArtifactRecord> {
        let content = ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: self.identity.digest.clone(),
        };
        store
            .put_file_expected_while(descriptor, path, &content, self.size_bytes, || {
                Instant::now() < deadline
            })
            .map_err(artifact_error)
    }
}

#[cfg(test)]
#[path = "output/tests.rs"]
mod tests;
