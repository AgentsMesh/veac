use std::path::{Path, PathBuf};
use std::time::Instant;

use tempfile::{Builder, TempDir};
use veac_artifact::{
    copy_verified_source_bounded_while, verify_source_bounded_while, ArtifactError,
    ArtifactErrorKind, ContentDigest,
};
use veac_ir::{HashAlgorithm, MediaIdentity};

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) struct SourceSnapshot {
    _directory: TempDir,
    path: PathBuf,
}

impl SourceSnapshot {
    pub(super) fn capture(
        path: &Path,
        expected: &ContentDigest,
        max_bytes: u64,
        deadline: Instant,
    ) -> WorkflowResult<Self> {
        let identity = identity(expected)?;
        let directory = Builder::new().prefix(".veac-media-source-").tempdir()?;
        let mut snapshot = directory.path().join("source");
        if let Some(extension) = path.extension() {
            snapshot.set_extension(extension);
        }
        copy_verified_source_bounded_while(path, &snapshot, Some(&identity), max_bytes, || {
            Instant::now() < deadline
        })
        .map_err(source_error)?;
        readonly(&snapshot)?;
        Ok(Self {
            _directory: directory,
            path: snapshot,
        })
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }
}

pub(super) fn verify_until(
    path: &Path,
    expected: &ContentDigest,
    max_bytes: u64,
    deadline: Instant,
) -> WorkflowResult<()> {
    verify_while(path, expected, max_bytes, || Instant::now() < deadline)
}

pub(super) fn verify_while(
    path: &Path,
    expected: &ContentDigest,
    max_bytes: u64,
    guard: impl FnMut() -> bool,
) -> WorkflowResult<()> {
    let identity = identity(expected)?;
    verify_source_bounded_while(path, Some(&identity), max_bytes, guard).map_err(source_error)?;
    Ok(())
}

fn identity(expected: &ContentDigest) -> WorkflowResult<MediaIdentity> {
    expected.validate()?;
    Ok(MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: expected.value.clone(),
    })
}

fn source_error(error: ArtifactError) -> WorkflowError {
    let kind = match error.kind {
        ArtifactErrorKind::IdentityMismatch => WorkflowErrorKind::SourceIdentityMismatch,
        ArtifactErrorKind::InvalidContract
        | ArtifactErrorKind::UnsupportedIdentity
        | ArtifactErrorKind::UnsafePath => WorkflowErrorKind::InvalidContract,
        ArtifactErrorKind::ResourceLimit => WorkflowErrorKind::ResourceLimit,
        _ => WorkflowErrorKind::Artifact,
    };
    WorkflowError::with_source(kind, "artifact source snapshot validation failed", error)
}

fn readonly(path: &Path) -> WorkflowResult<()> {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}
