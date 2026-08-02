use std::io::Write;
use std::path::Path;

use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest, OwnedStagedFile, StagedContent,
};

pub(super) enum Publish {
    Replace,
    NoClobber,
}

pub(super) fn bytes(
    destination: &Path,
    bytes: &[u8],
    publish: Publish,
    verify: impl FnOnce(&StagedContent) -> ArtifactResult<()>,
) -> ArtifactResult<()> {
    let parent = destination.parent().unwrap_or(Path::new("."));
    let mut staged = OwnedStagedFile::new_in(parent)?;
    let result = (|| {
        staged.file_mut().write_all(bytes)?;
        staged.file_mut().sync_all()?;
        let sealed = staged.seal()?;
        let expected = ContentDigest::sha256(bytes);
        if sealed.sha256 != expected.value || sealed.size_bytes != bytes.len() as u64 {
            return Err(ArtifactError::new(
                ArtifactErrorKind::IdentityMismatch,
                "staged package output differs from the requested bytes",
            ));
        }
        verify(&sealed)
    })();
    if let Err(error) = result {
        return Err(staged.discard_after(error));
    }
    match publish {
        Publish::Replace => staged.persist_replace(destination)?,
        Publish::NoClobber => staged.persist_noclobber(destination)?,
    };
    Ok(())
}
