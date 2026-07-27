use std::fs::File;
use std::path::Path;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, StagedContent};

#[derive(Debug, Clone, Copy)]
pub(in crate::staged) enum PublishMode {
    Replace,
    NoClobber,
}

#[derive(Debug)]
pub(in crate::staged) struct Stage;

impl Stage {
    pub(in crate::staged) fn new(_: &Path) -> ArtifactResult<Self> {
        Err(unsupported())
    }

    pub(in crate::staged) fn path(&self) -> &Path {
        unreachable!("unsupported staging cannot be constructed")
    }

    pub(in crate::staged) fn file_mut(&mut self) -> &mut File {
        unreachable!("unsupported staging cannot be constructed")
    }

    pub(in crate::staged) fn verify_payload(&self) -> ArtifactResult<()> {
        Err(unsupported())
    }

    pub(in crate::staged) fn content_while(
        &self,
        _: impl FnMut() -> bool,
    ) -> ArtifactResult<StagedContent> {
        Err(unsupported())
    }

    pub(in crate::staged) fn publish_while(
        &mut self,
        _: &Path,
        _: PublishMode,
        _: &StagedContent,
        _: impl FnOnce(&Path),
        _: impl FnOnce(&Path),
        _: impl FnMut() -> bool,
    ) -> ArtifactResult<()> {
        Err(unsupported())
    }

    pub(in crate::staged) fn finish(&mut self) -> ArtifactResult<()> {
        Err(unsupported())
    }

    pub(in crate::staged) fn discard(&mut self) -> ArtifactResult<()> {
        Err(unsupported())
    }

    pub(in crate::staged) fn into_file(self) -> File {
        unreachable!("unsupported staging cannot be constructed")
    }
}

fn unsupported() -> ArtifactError {
    ArtifactError::new(
        ArtifactErrorKind::UnsafePath,
        "identity-bound staged publication is unsupported on this platform",
    )
}
