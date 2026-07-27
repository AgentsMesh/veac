use std::path::PathBuf;
use std::time::Instant;

use tempfile::{Builder, TempDir};
use veac_codegen::emitter::{
    BackendAction, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};

use super::deadline;
use super::output;
use super::process::FfmpegEnvironment;
use crate::RuntimeError;

mod commit;
pub(super) mod directory;
mod ffmpeg;
mod finish;
mod journal;
mod recovery;
mod target;
mod write;

pub(super) use target::{common_parent, common_parent_from_files};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub(super) struct StagedFile {
    pub source: PathBuf,
    pub target: PathBuf,
}

pub(super) struct StagedTask {
    directory: TempDir,
    descriptor: directory::Directory,
    files: Vec<StagedFile>,
    stale: Vec<PathBuf>,
    allow_empty: bool,
}

impl StagedTask {
    pub fn files(&self) -> &[StagedFile] {
        &self.files
    }

    pub fn targets(&self) -> Vec<PathBuf> {
        self.files
            .iter()
            .map(|value| value.target.clone())
            .collect()
    }

    pub fn commit(
        self,
        locks: &super::locking::OutputLocks,
        deadline: Instant,
    ) -> Result<(), RuntimeError> {
        deadline::ensure(deadline)?;
        let parent = common_parent_from_files(&self.files)?;
        let output = locks.directory_until(parent, deadline)?;
        let result = commit::apply_locked(
            commit::CommitContext::new(
                self.directory.path(),
                &self.descriptor,
                &output,
                &self.files,
                &self.stale,
                self.allow_empty,
            ),
            deadline,
        );
        self.finish_commit(result, &output, deadline)
    }
}

pub(super) fn perform(
    environment: &dyn FfmpegEnvironment,
    task: &BackendTask,
    deadline: Instant,
) -> Result<StagedTask, RuntimeError> {
    deadline::ensure(deadline)?;
    let parent = common_parent(&task.output)?;
    let directory = Builder::new()
        .prefix(".veac-stage-")
        .tempdir_in(parent)
        .map_err(|error| {
            RuntimeError::new(format!("cannot create render staging area: {error}"))
        })?;
    let descriptor = directory::Directory::open(directory.path())?;
    let (files, stale) = match &task.action {
        BackendAction::WriteFile { content, .. } => {
            write::stage(&directory, &descriptor, task, content, deadline)?
        }
        BackendAction::Ffmpeg(command) => {
            let mut command = command.clone();
            match &task.output {
                BackendOutput::ImageSequence { pattern } => ffmpeg::stage_sequence(
                    environment,
                    &directory,
                    &descriptor,
                    &mut command,
                    pattern,
                    deadline,
                )?,
                BackendOutput::File(path) if task.phase == BackendPhase::FirstPass => {
                    ffmpeg::stage_passlog(
                        environment,
                        &directory,
                        &descriptor,
                        task,
                        &mut command,
                        path,
                        deadline,
                    )?
                }
                BackendOutput::File(path) => {
                    let source = directory.path().join("payload");
                    command.output_path = source.clone();
                    ffmpeg::execute(environment, &directory, &descriptor, &command, deadline)?;
                    (
                        vec![StagedFile {
                            source,
                            target: path.clone(),
                        }],
                        Vec::new(),
                    )
                }
                BackendOutput::Files { paths } => ffmpeg::stage_files(
                    environment,
                    &directory,
                    &descriptor,
                    &mut command,
                    paths,
                    deadline,
                )?,
            }
        }
    };
    for file in &files {
        output::file_state_until(
            &file.source,
            task.product == BackendProduct::CaptionSidecar,
            deadline,
        )?;
    }
    if files.is_empty() {
        return Err(RuntimeError::new("backend task produced no output files"));
    }
    Ok(StagedTask {
        directory,
        descriptor,
        files,
        stale,
        allow_empty: task.product == BackendProduct::CaptionSidecar,
    })
}

pub(super) fn recover(
    locks: &super::locking::OutputLocks,
    parents: &[PathBuf],
    deadline: Instant,
) -> Result<(), RuntimeError> {
    recovery::recover_until(locks, parents, deadline)
}
