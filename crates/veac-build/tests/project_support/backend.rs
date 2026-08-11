use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use veac_build::*;

#[derive(Clone, Copy)]
pub enum BackendMode {
    Good,
    Fail,
    Cancelled,
    Missing,
    Duplicate,
    Escape,
    WrongId,
    Directory,
    Symlink,
    Cancel,
}

#[derive(Clone)]
pub struct TestBackend {
    pub calls: Arc<AtomicUsize>,
    pub mode: BackendMode,
    identity: ContentDigest,
}

impl TestBackend {
    pub fn new(mode: BackendMode) -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            mode,
            identity: ContentDigest::sha256(b"project-test-backend-v1"),
        }
    }

    pub fn with_identity(mode: BackendMode, identity: &[u8]) -> Self {
        Self {
            identity: ContentDigest::sha256(identity),
            ..Self::new(mode)
        }
    }

    pub fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ProjectBackend for TestBackend {
    fn implementation_identity(
        &self,
        action: ProjectActionKind,
    ) -> Result<ProjectBackendIdentity, ProjectBackendError> {
        let value = self.identity.clone();
        Ok(match action {
            ProjectActionKind::VeacRender => ProjectBackendIdentity::VeacRender {
                project_backend: value.clone(),
                build: value.clone(),
                compiler: value.clone(),
                codegen: value.clone(),
                runtime: value.clone(),
                ffmpeg: value,
            },
            ProjectActionKind::MediaDerivation => ProjectBackendIdentity::MediaDerivation {
                project_backend: value.clone(),
                build: value.clone(),
                codegen: value.clone(),
                runtime: value.clone(),
                ffmpeg: value.clone(),
                ffprobe: value,
            },
            ProjectActionKind::Evidence => ProjectBackendIdentity::Evidence {
                project_backend: value.clone(),
                build: value.clone(),
                compiler: value.clone(),
                evidence: value.clone(),
                runtime: value.clone(),
                ffmpeg: value.clone(),
                ffprobe: value,
            },
        })
    }

    fn execute(
        &self,
        request: ProjectExecutionRequest<'_>,
        cancellation: &CancellationToken,
    ) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if matches!(self.mode, BackendMode::Fail) {
            return Err(ProjectBackendError::failed("backend failed"));
        }
        if matches!(self.mode, BackendMode::Cancelled) {
            return Err(ProjectBackendError::cancelled("backend cancelled"));
        }
        let mut outputs = Vec::new();
        for output in &request.action.computation().outputs {
            let path = if matches!(output, veac_project::ProjectOutput::Directory { .. }) {
                let path = format!("{}.directory", output.id());
                std::fs::create_dir(request.workspace.join(&path)).unwrap();
                std::fs::write(
                    request.workspace.join(&path).join("result.txt"),
                    format!("{}:{}", request.node_id, request.inputs.len()),
                )
                .unwrap();
                path
            } else {
                let path = format!("{}.bin", output.id());
                std::fs::write(
                    request.workspace.join(&path),
                    format!("{}:{}", request.node_id, request.inputs.len()),
                )
                .unwrap();
                path
            };
            outputs.push(ProducedProjectOutput {
                output: output.id().clone(),
                relative_path: PathBuf::from(path),
                semantics: ProjectOutputSemantics::Opaque,
            });
        }
        match self.mode {
            BackendMode::Missing => outputs.clear(),
            BackendMode::Duplicate => outputs.push(outputs[0].clone()),
            BackendMode::Escape => outputs[0].relative_path = PathBuf::from("../escape.bin"),
            BackendMode::WrongId => outputs[0].output = veac_project::OutputId::from("wrong"),
            BackendMode::Directory => {
                let path = request.workspace.join("directory");
                std::fs::create_dir(&path).unwrap();
                outputs[0].relative_path = PathBuf::from("directory");
            }
            BackendMode::Symlink => make_symlink(request.workspace, &mut outputs[0]),
            BackendMode::Cancel => cancellation.cancel(),
            BackendMode::Good | BackendMode::Fail | BackendMode::Cancelled => {}
        }
        Ok(outputs)
    }
}

fn make_symlink(workspace: &std::path::Path, output: &mut ProducedProjectOutput) {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(
            workspace.join(&output.relative_path),
            workspace.join("link.bin"),
        )
        .unwrap();
        output.relative_path = PathBuf::from("link.bin");
    }
    #[cfg(not(unix))]
    {
        output.relative_path = PathBuf::from("missing-symlink.bin");
    }
}
