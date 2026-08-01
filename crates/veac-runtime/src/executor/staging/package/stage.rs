use std::path::{Component, Path};
use std::time::Instant;

use tempfile::TempDir;
use veac_codegen::emitter::{BackendCommand, BackendPackagePaths};

use super::super::directory::Directory;
use super::super::{ffmpeg, StagedPackage};
use crate::executor::{output, FfmpegEnvironment};
use crate::RuntimeError;

pub(in crate::executor::staging) struct StageContext<'a> {
    environment: &'a dyn FfmpegEnvironment,
    directory: &'a TempDir,
    descriptor: &'a Directory,
    deadline: Instant,
}

impl<'a> StageContext<'a> {
    pub(in crate::executor::staging) fn new(
        environment: &'a dyn FfmpegEnvironment,
        directory: &'a TempDir,
        descriptor: &'a Directory,
        deadline: Instant,
    ) -> Self {
        Self {
            environment,
            directory,
            descriptor,
            deadline,
        }
    }
}

pub(in crate::executor::staging) fn stage(
    context: StageContext<'_>,
    command: &mut BackendCommand,
    target: &Path,
    entrypoint: &Path,
    paths: &BackendPackagePaths,
) -> Result<StagedPackage, RuntimeError> {
    let StageContext {
        environment,
        directory,
        descriptor,
        deadline,
    } = context;
    let segment = validate_layout(command, entrypoint, paths)?;
    let root = directory.path().join("package");
    let package = descriptor.create_child("package")?;
    command.output_path = root.join(&paths.playlist_pattern);
    command.output_args[segment + 1] = output::path_string(&root.join(&paths.segment_pattern));
    ffmpeg::execute(environment, directory, descriptor, command, deadline)?;
    package.sync()?;
    let inventory = super::inspect_hls(&root, entrypoint, deadline)?;
    Ok(StagedPackage {
        source: root,
        target: target.to_path_buf(),
        entrypoint: entrypoint.to_path_buf(),
        inventory,
    })
}

fn validate_layout(
    command: &BackendCommand,
    entrypoint: &Path,
    paths: &BackendPackagePaths,
) -> Result<usize, RuntimeError> {
    relative_leaf(entrypoint, "package entrypoint")?;
    relative_leaf(&paths.playlist_pattern, "package playlist pattern")?;
    relative_leaf(&paths.segment_pattern, "package segment pattern")?;
    if command.output_path != paths.playlist_pattern {
        return invalid("HLS command output does not match its package playlist pattern");
    }
    require_option(
        &command.output_args,
        "-master_pl_name",
        &output::path_string(entrypoint),
    )?;
    let segment = option_index(
        &command.output_args,
        "-hls_segment_filename",
        &output::path_string(&paths.segment_pattern),
    )?;
    Ok(segment)
}

fn require_option(arguments: &[String], name: &str, expected: &str) -> Result<(), RuntimeError> {
    option_index(arguments, name, expected).map(|_| ())
}

fn option_index(arguments: &[String], name: &str, expected: &str) -> Result<usize, RuntimeError> {
    let matches = arguments
        .windows(2)
        .enumerate()
        .filter(|(_, pair)| pair[0] == name && pair[1] == expected)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [index] => Ok(*index),
        _ => invalid("HLS command does not match its typed package layout"),
    }
}

fn relative_leaf(path: &Path, name: &str) -> Result<(), RuntimeError> {
    let mut components = path.components();
    if matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none() {
        Ok(())
    } else {
        Err(RuntimeError::new(format!(
            "{name} must be one safe relative component"
        )))
    }
}

fn invalid<T>(message: &str) -> Result<T, RuntimeError> {
    Err(RuntimeError::new(message))
}

#[cfg(test)]
#[path = "stage/tests.rs"]
mod tests;
