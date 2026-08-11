use serde::Serialize;
use veac_artifact::ContentDigest;

use crate::{BuildError, BuildResult, ProjectActionKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
/// Host-issued implementation identity for each executable project action.
pub enum ProjectBackendIdentity {
    VeacRender {
        project_backend: ContentDigest,
        build: ContentDigest,
        compiler: ContentDigest,
        codegen: ContentDigest,
        runtime: ContentDigest,
        ffmpeg: ContentDigest,
    },
    MediaDerivation {
        project_backend: ContentDigest,
        build: ContentDigest,
        codegen: ContentDigest,
        runtime: ContentDigest,
        ffmpeg: ContentDigest,
        ffprobe: ContentDigest,
    },
    Evidence {
        project_backend: ContentDigest,
        build: ContentDigest,
        compiler: ContentDigest,
        evidence: ContentDigest,
        runtime: ContentDigest,
        ffmpeg: ContentDigest,
        ffprobe: ContentDigest,
    },
}

impl ProjectBackendIdentity {
    pub fn kind(&self) -> ProjectActionKind {
        match self {
            Self::VeacRender { .. } => ProjectActionKind::VeacRender,
            Self::MediaDerivation { .. } => ProjectActionKind::MediaDerivation,
            Self::Evidence { .. } => ProjectActionKind::Evidence,
        }
    }

    pub fn digest_for(&self, action: ProjectActionKind) -> BuildResult<ContentDigest> {
        if self.kind() != action {
            return Err(BuildError::invalid(
                "project backend identity does not match the action",
            ));
        }
        for digest in self.digests() {
            digest.validate().map_err(|error| {
                BuildError::invalid(format!("project backend identity is invalid: {error}"))
            })?;
        }
        let bytes = serde_json_canonicalizer::to_vec(self).map_err(|error| {
            BuildError::invalid(format!(
                "project backend identity cannot be encoded: {error}"
            ))
        })?;
        Ok(ContentDigest::sha256(bytes))
    }

    fn digests(&self) -> Vec<&ContentDigest> {
        match self {
            Self::VeacRender {
                project_backend,
                build,
                compiler,
                codegen,
                runtime,
                ffmpeg,
            } => vec![project_backend, build, compiler, codegen, runtime, ffmpeg],
            Self::MediaDerivation {
                project_backend,
                build,
                codegen,
                runtime,
                ffmpeg,
                ffprobe,
            } => vec![project_backend, build, codegen, runtime, ffmpeg, ffprobe],
            Self::Evidence {
                project_backend,
                build,
                compiler,
                evidence,
                runtime,
                ffmpeg,
                ffprobe,
            } => vec![
                project_backend,
                build,
                compiler,
                evidence,
                runtime,
                ffmpeg,
                ffprobe,
            ],
        }
    }
}

#[cfg(test)]
#[path = "identity/tests.rs"]
mod tests;
