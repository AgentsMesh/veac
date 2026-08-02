use std::path::Path;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest};

pub(crate) fn write_project(
    root: &Path,
    bytes: &[u8],
    identity: &ContentDigest,
) -> ArtifactResult<()> {
    let destination = super::prepare_output_file(root, "project.veac.json")?;
    if super::existing_regular_file(&destination)? {
        return verify_digest_file(&destination, identity);
    }
    super::write::bytes(
        &destination,
        bytes,
        super::write::Publish::NoClobber,
        |content| {
            identity.validate()?;
            if content.sha256 == identity.value {
                Ok(())
            } else {
                Err(ArtifactError::new(
                    ArtifactErrorKind::IdentityMismatch,
                    "package file does not match its content digest",
                ))
            }
        },
    )
}

pub(crate) fn verify_digest_file(path: &Path, expected: &ContentDigest) -> ArtifactResult<()> {
    super::require_regular_file(path)?;
    expected.validate()?;
    if super::hash_file(path)? == expected.value {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::IdentityMismatch,
            "package file does not match its content digest",
        ))
    }
}
