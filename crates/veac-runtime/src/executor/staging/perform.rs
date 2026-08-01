use std::path::Path;
use std::time::Instant;

use veac_codegen::emitter::{
    BackendAction, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};

use super::directory::Directory;
use super::{
    common_parent, ffmpeg, ownership, package, write, StagedFile, StagedOutput, StagedTask,
};
use crate::executor::{deadline, output, FfmpegEnvironment};
use crate::RuntimeError;

pub(in crate::executor) fn perform(
    environment: &dyn FfmpegEnvironment,
    task: &BackendTask,
    deadline: Instant,
) -> Result<StagedTask, RuntimeError> {
    deadline::ensure(deadline)?;
    let parent = common_parent(&task.output)?;
    let (directory, descriptor) = ownership::create(parent, "render staging area")?;
    let (outputs, stale) = match &task.action {
        BackendAction::WriteFile { content, .. } => {
            let (files, stale) = write::stage(&directory, &descriptor, task, content, deadline)?;
            (file_outputs(files), stale)
        }
        BackendAction::Ffmpeg(command) => {
            let mut command = command.clone();
            stage_ffmpeg(
                environment,
                &directory,
                &descriptor,
                task,
                &mut command,
                deadline,
            )?
        }
    };
    for value in &outputs {
        match value {
            StagedOutput::File(file) => {
                output::file_state_until(&file.source, file.allow_empty, deadline)?;
            }
            StagedOutput::Package(package) => package
                .inventory
                .validate()
                .map_err(|error| RuntimeError::new(format!("invalid staged package: {error}")))?,
        }
    }
    if outputs.is_empty() {
        return Err(RuntimeError::new("backend task produced no outputs"));
    }
    Ok(StagedTask {
        directory,
        descriptor,
        outputs,
        stale,
    })
}

fn stage_ffmpeg(
    environment: &dyn FfmpegEnvironment,
    directory: &tempfile::TempDir,
    descriptor: &Directory,
    task: &BackendTask,
    command: &mut veac_codegen::emitter::BackendCommand,
    deadline: Instant,
) -> Result<(Vec<StagedOutput>, Vec<super::StaleFamily>), RuntimeError> {
    let internal_root = directory.path().join("preparations");
    if !command.preparations.is_empty() {
        std::fs::create_dir(&internal_root).map_err(staging_error)?;
    }
    for (index, preparation) in command.preparations.iter().enumerate() {
        let mut preparation = preparation.command.clone();
        for path in &command.preparations[index].outputs {
            let output = internal_root.join(path);
            let parent = output
                .parent()
                .expect("validated internal sidecar path has parent");
            std::fs::create_dir_all(parent).map_err(staging_error)?;
        }
        bind_internal(&mut preparation, &internal_root)?;
        preparation.output_path = directory.path().join(format!("preparation-{index}.null"));
        ffmpeg::execute(environment, directory, descriptor, &preparation, deadline)?;
        for path in &command.preparations[index].outputs {
            output::file_state_until(&internal_root.join(path), false, deadline)?;
        }
    }
    bind_internal(command, &internal_root)?;
    match &task.output {
        BackendOutput::ImageSequence { pattern } => {
            let (files, stale) = ffmpeg::stage_sequence(
                environment,
                directory,
                descriptor,
                command,
                pattern,
                deadline,
            )?;
            Ok((file_outputs(files), stale))
        }
        BackendOutput::File(path) if task.phase == BackendPhase::FirstPass => {
            let (files, stale) = ffmpeg::stage_passlog(
                environment,
                directory,
                descriptor,
                task,
                command,
                path,
                deadline,
            )?;
            Ok((file_outputs(files), stale))
        }
        BackendOutput::File(path) => {
            let source = directory.path().join("payload");
            command.output_path = source.clone();
            ffmpeg::execute(environment, directory, descriptor, command, deadline)?;
            Ok((
                vec![StagedOutput::File(StagedFile {
                    source,
                    target: path.clone(),
                    allow_empty: task.product == BackendProduct::CaptionSidecar,
                })],
                Vec::new(),
            ))
        }
        BackendOutput::Files { paths } => {
            let (files, stale) =
                ffmpeg::stage_files(environment, directory, descriptor, command, paths, deadline)?;
            Ok((file_outputs(files), stale))
        }
        BackendOutput::Package {
            root,
            entrypoint,
            paths,
        } => package::stage(
            package::StageContext::new(environment, directory, descriptor, deadline),
            command,
            root,
            entrypoint,
            paths,
        )
        .map(|package| (vec![StagedOutput::Package(package)], Vec::new())),
    }
}

fn bind_internal(
    command: &mut veac_codegen::emitter::BackendCommand,
    root: &Path,
) -> Result<(), RuntimeError> {
    let Some(contract) = &command.filter_contract else {
        return Ok(());
    };
    let graph = command
        .filter_graph
        .as_deref()
        .ok_or_else(|| RuntimeError::new("FFmpeg filter contract has no rendered graph"))?;
    command.filter_graph = Some(contract.render_internal(graph, root).map_err(|error| {
        RuntimeError::new(format!("cannot bind internal filter sidecar: {error}"))
    })?);
    Ok(())
}

fn file_outputs(files: Vec<StagedFile>) -> Vec<StagedOutput> {
    files.into_iter().map(StagedOutput::File).collect()
}

fn staging_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!("cannot prepare FFmpeg sidecar staging: {error}"))
}

#[cfg(test)]
#[path = "perform/tests.rs"]
mod tests;
