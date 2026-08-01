use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::Instant;

use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_codegen::emitter::{BackendAction, BackendOutput, BackendPhase, BackendTask};

use crate::RuntimeError;

mod dynamic;

#[cfg(test)]
mod tests;

pub(super) use dynamic::{enumerate_passlogs, enumerate_pattern};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FileState {
    pub content: ContentDigest,
    pub size_bytes: u64,
}

pub(super) fn current_paths(task: &BackendTask) -> Result<Vec<PathBuf>, RuntimeError> {
    match &task.output {
        BackendOutput::File(path) if task.phase == BackendPhase::FirstPass => {
            let prefix = passlog_prefix(task)?;
            let paths = enumerate_passlogs(&prefix)?;
            if paths.iter().any(|value| value == path) {
                Ok(paths)
            } else {
                Ok(Vec::new())
            }
        }
        BackendOutput::File(path) => Ok(vec![path.clone()]),
        BackendOutput::Files { paths } => {
            let mut paths = paths.clone();
            paths.sort_by_key(|path| path_string(path));
            Ok(paths)
        }
        BackendOutput::ImageSequence { pattern } => enumerate_pattern(pattern),
        BackendOutput::Package { root, .. } => Ok(vec![root.clone()]),
    }
}

pub(super) fn file_state_until(
    path: &Path,
    allow_empty: bool,
    deadline: Instant,
) -> Result<FileState, RuntimeError> {
    let verified = veac_artifact::verify_source_bounded_while(
        path,
        None,
        veac_artifact::MAX_RENDER_TASK_OUTPUT_BYTES,
        || Instant::now() < deadline,
    )
    .map_err(|error| output_error(path, error))?;
    if !allow_empty && verified.size_bytes == 0 {
        return Err(RuntimeError::new(format!(
            "render output {} must be a non-empty regular file",
            path.display()
        )));
    }
    Ok(FileState {
        content: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: verified.identity.digest,
        },
        size_bytes: verified.size_bytes,
    })
}

fn output_error(path: &Path, error: veac_artifact::ArtifactError) -> RuntimeError {
    let message = format!(
        "render output {} is unavailable or unsafe: {error}",
        path.display()
    );
    if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
        RuntimeError::resource_limit(message)
    } else {
        RuntimeError::new(message)
    }
}

pub(super) fn ensure_safe_parent(path: &Path) -> Result<&Path, RuntimeError> {
    if path
        .components()
        .any(|value| matches!(value, Component::ParentDir))
    {
        return Err(RuntimeError::new(format!(
            "render output path may not contain '..': {}",
            path.display()
        )));
    }
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let metadata = fs::symlink_metadata(parent).map_err(|error| {
        RuntimeError::new(format!(
            "render output directory {} is unavailable: {error}",
            parent.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(RuntimeError::new(format!(
            "render output parent {} must be a non-symlink directory",
            parent.display()
        )));
    }
    if path.file_name().is_none() {
        return Err(RuntimeError::new("render output path has no file name"));
    }
    Ok(parent)
}

pub(super) fn passlog_prefix(task: &BackendTask) -> Result<PathBuf, RuntimeError> {
    let BackendAction::Ffmpeg(command) = &task.action else {
        return Err(RuntimeError::new(
            "first-pass task must be an FFmpeg action",
        ));
    };
    let values: Vec<_> = command
        .output_args
        .windows(2)
        .filter(|pair| pair[0] == "-passlogfile")
        .map(|pair| PathBuf::from(&pair[1]))
        .collect();
    match values.as_slice() {
        [value] => Ok(value.clone()),
        _ => Err(RuntimeError::new(
            "first-pass task requires one passlog prefix",
        )),
    }
}

pub(super) fn appended(path: &Path, suffix: &str) -> PathBuf {
    let mut value: OsString = path.as_os_str().to_owned();
    value.push(suffix);
    value.into()
}

pub(super) fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
