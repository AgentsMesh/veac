use std::path::PathBuf;

use veac_artifact::{ArtifactStore, ContentDigest};
use veac_build::{
    CancellationToken, ProducedProjectOutput, ProjectBackend, ProjectBackendError,
    ProjectBackendIdentity, ProjectExecutionRequest,
};
use veac_runtime::asset::SystemFfprobe;
use veac_runtime::executor::{FfmpegEnvironment, SystemFfmpeg};

mod derivation;
mod error;
mod evidence;
mod identity;
mod inputs;
mod render;

use error::{ProjectOptionExt, ProjectResultExt};

pub(super) struct CliProjectBackend {
    source_root: PathBuf,
    material_root: PathBuf,
    packages: veac_build::ProjectPackageSet,
    store: ArtifactStore,
    manifest_digest: ContentDigest,
    ffmpeg: SystemFfmpeg,
    ffprobe: SystemFfprobe,
}

impl CliProjectBackend {
    pub(super) fn new(
        source_root: PathBuf,
        material_root: PathBuf,
        packages: veac_build::ProjectPackageSet,
        store: ArtifactStore,
        manifest_digest: ContentDigest,
    ) -> Self {
        Self::with_tools(
            source_root,
            material_root,
            packages,
            store,
            manifest_digest,
            SystemFfmpeg::default(),
            SystemFfprobe::default(),
        )
    }

    fn with_tools(
        source_root: PathBuf,
        material_root: PathBuf,
        packages: veac_build::ProjectPackageSet,
        store: ArtifactStore,
        manifest_digest: ContentDigest,
        ffmpeg: SystemFfmpeg,
        ffprobe: SystemFfprobe,
    ) -> Self {
        Self {
            source_root,
            material_root,
            packages,
            store,
            manifest_digest,
            ffmpeg,
            ffprobe,
        }
    }
}

impl ProjectBackend for CliProjectBackend {
    fn implementation_identity(
        &self,
        action: &veac_build::ProjectAction,
    ) -> Result<ProjectBackendIdentity, ProjectBackendError> {
        self.verify_packages(action)?;
        let ffmpeg = || {
            FfmpegEnvironment::fingerprint(&self.ffmpeg)
                .map(|value| value.configuration)
                .map_err(|error| failed(format!("project FFmpeg identity failed: {error}")))
        };
        let ffprobe = || {
            self.ffprobe
                .fingerprint()
                .map(|value| value.configuration)
                .map_err(|error| failed(format!("project ffprobe identity failed: {error}")))
        };
        let identity = match action.kind() {
            veac_build::ProjectActionKind::VeacRender => identity::render(ffmpeg()?),
            veac_build::ProjectActionKind::MediaDerivation => {
                identity::derivation(ffmpeg()?, ffprobe()?)
            }
            veac_build::ProjectActionKind::Evidence => identity::evidence(ffmpeg()?, ffprobe()?),
        };
        self.verify_packages_after(action)?;
        Ok(identity)
    }

    fn execute(
        &self,
        request: ProjectExecutionRequest<'_>,
        cancellation: &CancellationToken,
    ) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError> {
        self.verify_packages(request.action)?;
        if cancellation.is_cancelled() {
            return Err(ProjectBackendError::cancelled(
                "project execution was cancelled before launch",
            ));
        }
        let produced = match request.action {
            veac_build::ProjectAction::VeacRender {
                computation,
                source,
                source_graph,
            } => render::execute(
                self,
                request.workspace,
                computation,
                source,
                source_graph,
                request.inputs,
                cancellation,
            )?,
            veac_build::ProjectAction::MediaDerivation {
                computation,
                operation,
            } => derivation::execute(
                self,
                request.workspace,
                computation,
                operation,
                request.inputs,
                cancellation,
            )?,
            veac_build::ProjectAction::Evidence {
                computation,
                contract,
                source_graph,
            } => evidence::execute(
                self,
                request.workspace,
                computation,
                contract,
                source_graph,
                request.inputs,
                cancellation,
            )?,
        };
        self.verify_packages_after(request.action)?;
        Ok(produced)
    }
}

impl CliProjectBackend {
    fn verify_packages(
        &self,
        action: &veac_build::ProjectAction,
    ) -> Result<(), ProjectBackendError> {
        self.packages
            .revalidate()
            .and_then(|()| {
                self.packages
                    .require_revision(&action.computation().package_mounts)
            })
            .map_err(|error| failed(format!("project package verification failed: {error}")))
    }

    fn verify_packages_after(
        &self,
        action: &veac_build::ProjectAction,
    ) -> Result<(), ProjectBackendError> {
        self.verify_packages(action)
    }
}

fn failed(message: impl Into<String>) -> ProjectBackendError {
    ProjectBackendError::failed(message)
}

#[cfg(test)]
#[path = "backend/package_tests.rs"]
mod package_tests;
