use std::path::PathBuf;
use std::time::Instant;

use veac_artifact::{
    artifact_key, ArtifactErrorKind, ArtifactRecord, ArtifactStore, ContentDigest,
};
use veac_codegen::emitter::{BackendOutput, BackendProduct, BackendTask};

use super::identity::TaskIdentity;
use super::manifest::{self, CheckpointOutput};
use super::model::{package_record_matches, ResumeHit};
use crate::executor::{deadline, output, staging};
use crate::RuntimeError;

pub(in crate::executor) fn resume(
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
        Err(error) => {
            return Err(super::error::artifact(
                "cannot read render checkpoint",
                error,
            ))
        }
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

pub(in crate::executor) fn invalidate(
    store: &ArtifactStore,
    key: &ContentDigest,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    store
        .remove_while(key, || Instant::now() < deadline)
        .map(|_| ())
        .map_err(|error| super::error::artifact("cannot invalidate render checkpoint", error))
}

pub(in crate::executor) fn validate_cached(
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
        validate_entry(task, identity, path, entry, index, deadline)?;
    }
    Ok((
        paths,
        manifest
            .outputs
            .into_iter()
            .map(|value| value.into_record())
            .collect(),
    ))
}

fn validate_entry(
    task: &BackendTask,
    identity: &TaskIdentity,
    path: &std::path::Path,
    entry: &CheckpointOutput,
    index: usize,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    if output::path_string(path) != entry.path() {
        return Err(RuntimeError::new("checkpoint output path changed"));
    }
    let expected = super::identity::output(identity, task.product, entry.path(), index);
    if entry.descriptor() != &expected
        || entry.record().key
            != artifact_key(&expected)
                .map_err(|error| super::error::artifact("invalid output descriptor", error))?
    {
        return Err(RuntimeError::new("checkpoint output descriptor changed"));
    }
    match entry {
        CheckpointOutput::File { record, .. } => {
            let state = output::file_state_until(
                path,
                task.product == BackendProduct::CaptionSidecar,
                deadline,
            )?;
            if state.content == record.content && state.size_bytes == record.size_bytes {
                Ok(())
            } else {
                Err(RuntimeError::new("checkpoint output content changed"))
            }
        }
        CheckpointOutput::Package {
            entrypoint,
            inventory,
            record,
            ..
        } => validate_package(task, path, entrypoint, inventory, record, deadline),
    }
}

fn validate_package(
    task: &BackendTask,
    root: &std::path::Path,
    entrypoint: &str,
    inventory: &veac_artifact::DeliveryPackageInventory,
    record: &ArtifactRecord,
    deadline: Instant,
) -> Result<(), RuntimeError> {
    let BackendOutput::Package {
        entrypoint: declared,
        ..
    } = &task.output
    else {
        return Err(RuntimeError::new("checkpoint output kind changed"));
    };
    if output::path_string(declared) != entrypoint || !package_record_matches(record, inventory) {
        return Err(RuntimeError::new("checkpoint package contract changed"));
    }
    let current = staging::inspect_package(root, declared, deadline)?;
    if &current == inventory {
        Ok(())
    } else {
        Err(RuntimeError::new("checkpoint package content changed"))
    }
}
