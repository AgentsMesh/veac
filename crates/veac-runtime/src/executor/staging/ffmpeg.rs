use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tempfile::TempDir;
use veac_codegen::emitter::{BackendCommand, BackendTask};

use super::directory::Directory;
use super::StagedFile;
use crate::executor::{output, process, FfmpegEnvironment, FfmpegInvocation};
use crate::RuntimeError;

mod filter_script;

pub(super) fn execute(
    environment: &dyn FfmpegEnvironment,
    directory: &TempDir,
    descriptor: &Directory,
    command: &BackendCommand,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    crate::executor::deadline::ensure(deadline)?;
    let script = filter_script::stage(directory.path(), descriptor, command)?;
    let arguments = match &script {
        Some(script) => process::script_arguments(command, &script.path),
        None => process::arguments(command),
    };
    let output_root = command
        .output_path
        .parent()
        .ok_or_else(|| RuntimeError::new("staged FFmpeg output has no resource-bounded parent"))?;
    let result = environment.execute(FfmpegInvocation::render(&arguments, output_root, deadline));
    filter_script::finish(descriptor, script, result)
}

pub(super) fn stage_sequence(
    environment: &dyn FfmpegEnvironment,
    directory: &TempDir,
    descriptor: &Directory,
    command: &mut BackendCommand,
    pattern: &Path,
    deadline: Instant,
) -> Result<(Vec<StagedFile>, Vec<PathBuf>), RuntimeError> {
    let staged_pattern = directory.path().join(pattern.file_name().unwrap());
    command.output_path = staged_pattern.clone();
    execute(environment, directory, descriptor, command, deadline)?;
    let parent = pattern
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let files = output::enumerate_pattern(&staged_pattern)?
        .into_iter()
        .map(|source| StagedFile {
            target: parent.join(source.file_name().unwrap()),
            source,
        })
        .collect::<Vec<_>>();
    let stale = stale(output::enumerate_pattern(pattern)?, &files);
    Ok((files, stale))
}

pub(super) fn stage_passlog(
    environment: &dyn FfmpegEnvironment,
    directory: &TempDir,
    descriptor: &Directory,
    task: &BackendTask,
    command: &mut BackendCommand,
    expected: &Path,
    deadline: Instant,
) -> Result<(Vec<StagedFile>, Vec<PathBuf>), RuntimeError> {
    let final_prefix = output::passlog_prefix(task)?;
    let staged_prefix = directory.path().join("passlog");
    replace_passlog(&mut command.output_args, &staged_prefix);
    command.output_path = directory.path().join("first-pass.null");
    execute(environment, directory, descriptor, command, deadline)?;
    let files = output::enumerate_passlogs(&staged_prefix)?
        .into_iter()
        .map(|source| {
            let name = source.file_name().unwrap().to_string_lossy();
            let suffix = name.strip_prefix("passlog").unwrap_or("");
            StagedFile {
                target: output::appended(&final_prefix, suffix),
                source,
            }
        })
        .collect::<Vec<_>>();
    if !files.iter().any(|value| value.target == expected) {
        return Err(RuntimeError::new(
            "FFmpeg did not produce its declared passlog",
        ));
    }
    let stale = stale(output::enumerate_passlogs(&final_prefix)?, &files);
    Ok((files, stale))
}

pub(super) fn stage_files(
    environment: &dyn FfmpegEnvironment,
    directory: &TempDir,
    descriptor: &Directory,
    command: &mut BackendCommand,
    paths: &[PathBuf],
    deadline: Instant,
) -> Result<(Vec<StagedFile>, Vec<PathBuf>), RuntimeError> {
    command.output_path = directory.path().join(paths[0].file_name().unwrap());
    execute(environment, directory, descriptor, command, deadline)?;
    let files = paths
        .iter()
        .map(|target| StagedFile {
            source: directory.path().join(target.file_name().unwrap()),
            target: target.clone(),
        })
        .collect();
    Ok((files, Vec::new()))
}

fn replace_passlog(arguments: &mut [String], prefix: &Path) {
    if let Some(index) = arguments.iter().position(|value| value == "-passlogfile") {
        arguments[index + 1] = output::path_string(prefix);
    }
}

fn stale(existing: Vec<PathBuf>, files: &[StagedFile]) -> Vec<PathBuf> {
    let targets: BTreeSet<_> = files.iter().map(|value| &value.target).collect();
    existing
        .into_iter()
        .filter(|value| !targets.contains(value))
        .collect()
}
