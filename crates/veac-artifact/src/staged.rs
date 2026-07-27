use std::fs::File;
use std::path::Path;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

mod platform;

use platform::{PublishMode, Stage};

#[derive(Debug)]
pub struct OwnedStagedFile {
    inner: Option<Stage>,
    sealed: Option<StagedContent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedContent {
    pub sha256: String,
    pub size_bytes: u64,
}

impl OwnedStagedFile {
    pub fn new_in(parent: &Path) -> ArtifactResult<Self> {
        Ok(Self {
            inner: Some(Stage::new(parent)?),
            sealed: None,
        })
    }

    pub fn path(&self) -> &Path {
        self.inner().path()
    }

    pub fn file_mut(&mut self) -> &mut File {
        self.sealed = None;
        self.inner_mut().file_mut()
    }

    pub fn verify_path(&self) -> ArtifactResult<()> {
        self.inner().verify_payload()
    }

    pub fn seal(&mut self) -> ArtifactResult<StagedContent> {
        self.seal_while(|| true)
    }

    pub fn seal_while(&mut self, guard: impl FnMut() -> bool) -> ArtifactResult<StagedContent> {
        let content = self.inner().content_while(guard)?;
        self.sealed = Some(content.clone());
        Ok(content)
    }

    pub fn persist_replace(self, destination: &Path) -> ArtifactResult<File> {
        self.persist(destination, PublishMode::Replace, |_| {}, |_| {})
    }

    pub fn persist_noclobber(self, destination: &Path) -> ArtifactResult<File> {
        self.persist(destination, PublishMode::NoClobber, |_| {}, |_| {})
    }

    pub fn persist_noclobber_while(
        self,
        destination: &Path,
        guard: impl FnMut() -> bool,
    ) -> ArtifactResult<File> {
        self.persist_while(destination, PublishMode::NoClobber, |_| {}, |_| {}, guard)
    }

    pub fn discard(mut self) -> ArtifactResult<()> {
        self.take().discard()
    }

    pub(crate) fn discard_after(self, primary: ArtifactError) -> ArtifactError {
        match self.discard() {
            Ok(()) => primary,
            Err(cleanup) => primary.with_cleanup_failure(cleanup, "staged output cleanup failed"),
        }
    }

    fn persist(
        self,
        destination: &Path,
        mode: PublishMode,
        after_verify: impl FnOnce(&Path),
        after_publish: impl FnOnce(&Path),
    ) -> ArtifactResult<File> {
        self.persist_while(destination, mode, after_verify, after_publish, || true)
    }

    fn persist_while(
        mut self,
        destination: &Path,
        mode: PublishMode,
        after_verify: impl FnOnce(&Path),
        after_publish: impl FnOnce(&Path),
        mut guard: impl FnMut() -> bool,
    ) -> ArtifactResult<File> {
        check_guard(&mut guard)?;
        let sealed = self.sealed.take();
        let mut stage = self.take();
        let expected = match sealed {
            Some(content) => content,
            None => match stage.content_while(&mut guard) {
                Ok(content) => content,
                Err(primary) => return Err(cleanup_failure(&mut stage, primary)),
            },
        };
        if let Err(primary) = stage.publish_while(
            destination,
            mode,
            &expected,
            after_verify,
            after_publish,
            &mut guard,
        ) {
            return Err(cleanup_failure(&mut stage, primary));
        }
        let deadline = check_guard(&mut guard).map_err(ArtifactError::committed);
        let cleanup = stage.finish();
        if let Err(primary) = deadline {
            return Err(match cleanup {
                Ok(()) => primary,
                Err(error) => primary.with_cleanup_failure(
                    error,
                    "published output staging directory could not be removed",
                ),
            });
        }
        if let Err(cleanup) = cleanup {
            return Err(ArtifactError::with_source(
                ArtifactErrorKind::UnsafePath,
                "publication crossed the commit point but its staging directory could not be removed",
                cleanup,
            )
            .committed());
        }
        check_guard(&mut guard).map_err(ArtifactError::committed)?;
        Ok(stage.into_file())
    }

    fn inner(&self) -> &Stage {
        self.inner.as_ref().expect("staged file is live")
    }

    fn inner_mut(&mut self) -> &mut Stage {
        self.inner.as_mut().expect("staged file is live")
    }

    fn take(&mut self) -> Stage {
        self.inner.take().expect("staged file is live")
    }
}

impl Drop for OwnedStagedFile {
    fn drop(&mut self) {
        // Drop cannot return cleanup failures. Call `discard` when cleanup must be observed.
        if let Some(mut stage) = self.inner.take() {
            let _ = stage.discard();
        }
    }
}

fn cleanup_failure(stage: &mut Stage, primary: ArtifactError) -> ArtifactError {
    match stage.discard() {
        Ok(()) => primary,
        Err(cleanup) => {
            primary.with_cleanup_failure(cleanup, "staged output failed and cleanup was unsafe")
        }
    }
}

pub(super) fn check_guard(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    if guard() {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::ResourceLimit,
            "staged file operation exceeded its caller resource guard",
        ))
    }
}

#[cfg(test)]
mod tests;
