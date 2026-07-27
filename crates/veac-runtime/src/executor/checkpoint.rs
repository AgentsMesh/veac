use std::path::PathBuf;
use std::time::Instant;

use veac_artifact::{
    artifact_key, ArtifactErrorKind, ArtifactRecord, ArtifactStore, ContentDigest,
};
use veac_codegen::emitter::{BackendProduct, BackendTask};

use super::deadline;
use super::output;
use super::process::FfmpegFingerprint;
use super::staging::StagedFile;
use crate::RuntimeError;

mod cleanup;
mod identity;
mod manifest;

#[cfg(test)]
mod tests;

pub(super) use cleanup::after_failure;
pub(super) use identity::TaskIdentity;
use manifest::{CheckpointManifest, CheckpointOutput};

pub(super) struct ResumeHit {
    pub record: ArtifactRecord,
    pub paths: Vec<PathBuf>,
    pub output_records: Vec<ArtifactRecord>,
}

#[derive(Debug)]
pub(super) struct StoredCheckpoint {
    pub record: ArtifactRecord,
    pub output_records: Vec<ArtifactRecord>,
}

pub(super) fn identity(
    task: &BackendTask,
    plan: &ContentDigest,
    resources: &ContentDigest,
    ffmpeg: Option<&FfmpegFingerprint>,
    predecessor: Option<&ContentDigest>,
) -> Result<TaskIdentity, RuntimeError> {
    identity::task(task, plan, resources, ffmpeg, predecessor)
}

pub(super) fn resume(
    store: &ArtifactStore,
    task: &BackendTask,
    identity: &TaskIdentity,
    deadline: Instant,
) -> Result<Option<ResumeHit>, RuntimeError> {
    deadline::ensure(deadline)?;
    let cached = match store.get_while(&identity.key, || Instant::now() < deadline) {
        Ok(value) => value,
        Err(error) if error.kind == ArtifactErrorKind::CorruptCache => {
            invalidate(store, &identity.key, deadline)?;
            return Ok(None);
        }
        Err(error) => return Err(artifact_error("cannot read render checkpoint", error)),
    };
    let Some(cached) = cached else {
        return Ok(None);
    };
    let valid = (cached.descriptor == identity.descriptor)
        .then(|| validate_cached(task, identity, &cached.payload, deadline).ok())
        .flatten();
    if let Some((paths, output_records)) = valid {
        return Ok(Some(ResumeHit {
            record: cached.record,
            paths,
            output_records,
        }));
    }
    invalidate(store, &identity.key, deadline)?;
    Ok(None)
}

pub(super) fn store(
    store: &ArtifactStore,
    task: &BackendTask,
    identity: &TaskIdentity,
    files: &[StagedFile],
    deadline: Instant,
) -> Result<StoredCheckpoint, RuntimeError> {
    deadline::ensure(deadline)?;
    let mut files = files.to_vec();
    files.sort_by(|left, right| {
        output::path_string(&left.target).cmp(&output::path_string(&right.target))
    });
    let mut entries = Vec::with_capacity(files.len());
    for (index, file) in files.iter().enumerate() {
        let state = output::file_state_until(
            &file.source,
            task.product == BackendProduct::CaptionSidecar,
            deadline,
        )?;
        if state.size_bytes > veac_ir::MAX_SAFE_INTEGER {
            return Err(RuntimeError::new(
                "render output exceeds canonical size limit",
            ));
        }
        let path = output::path_string(&file.target);
        let descriptor = identity::output(identity, task.product, &path, index);
        entries.push(CheckpointOutput {
            path,
            record: ArtifactRecord {
                key: artifact_key(&descriptor)
                    .map_err(|error| artifact_error("cannot identify render output", error))?,
                content: state.content,
                size_bytes: state.size_bytes,
            },
            descriptor,
        });
    }
    let output_records = entries.iter().map(|value| value.record.clone()).collect();
    let payload = manifest::encode(&CheckpointManifest {
        schema_version: 1,
        outputs: entries,
    })?;
    deadline::ensure(deadline)?;
    let record = store
        .put_while(&identity.descriptor, &payload, || Instant::now() < deadline)
        .map_err(|error| artifact_error("cannot store render checkpoint", error))?;
    Ok(StoredCheckpoint {
        record,
        output_records,
    })
}

pub(super) fn invalidate(
    store: &ArtifactStore,
    key: &ContentDigest,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    store
        .remove_while(key, || Instant::now() < deadline)
        .map(|_| ())
        .map_err(|error| artifact_error("cannot invalidate render checkpoint", error))
}

fn validate_cached(
    task: &BackendTask,
    identity: &TaskIdentity,
    payload: &[u8],
    deadline: Instant,
) -> Result<(Vec<PathBuf>, Vec<ArtifactRecord>), RuntimeError> {
    deadline::ensure(deadline)?;
    let manifest = manifest::decode(payload)?;
    let paths = output::current_paths(task)?;
    if paths.len() != manifest.outputs.len() {
        return Err(RuntimeError::new("checkpoint output count changed"));
    }
    for (index, (path, entry)) in paths.iter().zip(&manifest.outputs).enumerate() {
        if output::path_string(path) != entry.path {
            return Err(RuntimeError::new("checkpoint output path changed"));
        }
        let expected = identity::output(identity, task.product, &entry.path, index);
        if entry.descriptor != expected
            || entry.record.key
                != artifact_key(&entry.descriptor)
                    .map_err(|error| artifact_error("invalid output descriptor", error))?
        {
            return Err(RuntimeError::new("checkpoint output descriptor changed"));
        }
        let state = output::file_state_until(
            path,
            task.product == BackendProduct::CaptionSidecar,
            deadline,
        )?;
        if state.content != entry.record.content || state.size_bytes != entry.record.size_bytes {
            return Err(RuntimeError::new("checkpoint output content changed"));
        }
    }
    let records = manifest
        .outputs
        .into_iter()
        .map(|value| value.record)
        .collect();
    Ok((paths, records))
}

fn artifact_error(context: &str, error: veac_artifact::ArtifactError) -> RuntimeError {
    let message = format!("{context}: {error}");
    if error.kind == ArtifactErrorKind::ResourceLimit {
        RuntimeError::resource_limit(message)
    } else {
        RuntimeError::new(message)
    }
}
