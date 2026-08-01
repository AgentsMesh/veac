use std::path::{Path, PathBuf};
use std::time::Instant;

use tempfile::{Builder, TempDir};
use veac_artifact::{
    copy_verified_source_bounded_while, ArtifactRecord, DigestAlgorithm,
    MAX_RENDER_TASK_OUTPUT_BYTES,
};
use veac_codegen::emitter::{BackendAction, BackendPhase, BackendTask};
use veac_ir::{HashAlgorithm, MediaIdentity};

use super::readonly;
use crate::executor::output;
use crate::RuntimeError;

#[derive(Debug)]
pub(in crate::executor) struct ReboundPasslogs {
    _directory: TempDir,
    task: BackendTask,
}

impl ReboundPasslogs {
    pub(in crate::executor) fn capture(
        task: &BackendTask,
        sources: &[PathBuf],
        targets: &[PathBuf],
        records: &[ArtifactRecord],
        deadline: Instant,
    ) -> Result<Self, RuntimeError> {
        if task.phase != BackendPhase::SecondPass
            || sources.is_empty()
            || sources.len() != targets.len()
            || sources.len() != records.len()
        {
            return Err(RuntimeError::new(
                "second pass requires complete checkpointed passlog outputs",
            ));
        }
        if !matches!(task.action, BackendAction::Ffmpeg(_)) {
            return Err(RuntimeError::new("second pass must be an FFmpeg action"));
        }
        let original = output::passlog_prefix(task)?;
        let directory = match Builder::new().prefix(".veac-passlogs-").tempdir() {
            Ok(directory) => directory,
            Err(error) => {
                return Err(RuntimeError::new(format!(
                    "cannot create private passlog snapshot: {error}"
                )))
            }
        };
        let private = directory.path().join("passlog");
        for ((source, target), record) in sources.iter().zip(targets).zip(records) {
            copy_one(source, target, record, &original, &private, deadline)?;
        }
        let mut task = task.clone();
        if let BackendAction::Ffmpeg(command) = &mut task.action {
            replace_prefix(&mut command.output_args, &private);
        }
        Ok(Self {
            _directory: directory,
            task,
        })
    }

    pub(in crate::executor) fn task(&self) -> &BackendTask {
        &self.task
    }
}

fn copy_one(
    source: &Path,
    target: &Path,
    record: &ArtifactRecord,
    original: &Path,
    private: &Path,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    let suffix = suffix(target, original)?;
    let destination = output::appended(private, suffix);
    let expected = MediaIdentity {
        algorithm: match record.content.algorithm {
            DigestAlgorithm::Sha256 => HashAlgorithm::Sha256,
        },
        digest: record.content.value.clone(),
    };
    let copied = copy_verified_source_bounded_while(
        source,
        &destination,
        Some(&expected),
        MAX_RENDER_TASK_OUTPUT_BYTES,
        || Instant::now() < deadline,
    )
    .map_err(|error| {
        let message = format!(
            "cannot snapshot checkpointed passlog {}: {error}",
            source.display()
        );
        if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
            RuntimeError::resource_limit(message)
        } else {
            RuntimeError::new(message)
        }
    })?;
    if copied.size_bytes != record.size_bytes {
        return Err(RuntimeError::new(format!(
            "checkpointed passlog size changed: {}",
            source.display()
        )));
    }
    readonly(&destination)
}

fn suffix<'a>(path: &'a Path, prefix: &Path) -> Result<&'a str, RuntimeError> {
    let name = path.file_name().and_then(|value| value.to_str());
    let prefix = prefix.file_name().and_then(|value| value.to_str());
    match (name, prefix) {
        (Some(name), Some(prefix)) => name
            .strip_prefix(prefix)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| RuntimeError::new("checkpoint output is not in the passlog family")),
        _ => Err(RuntimeError::new("passlog names must be valid UTF-8")),
    }
}

fn replace_prefix(arguments: &mut [String], prefix: &Path) {
    if let Some(index) = arguments.iter().position(|value| value == "-passlogfile") {
        arguments[index + 1] = output::path_string(prefix);
    }
}

#[cfg(test)]
mod tests;
