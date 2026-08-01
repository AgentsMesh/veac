use std::time::{Instant, Instant as Deadline};

use veac_artifact::{artifact_key, ArtifactRecord, ArtifactStore};
use veac_codegen::emitter::BackendTask;

use super::identity::TaskIdentity;
use super::manifest::{self, CheckpointManifest, CheckpointOutput};
use super::model::StoredCheckpoint;
use crate::executor::staging::StagedOutput;
use crate::executor::{deadline, output, staging};
use crate::RuntimeError;

pub(in crate::executor) fn store<T>(
    store: &ArtifactStore,
    task: &BackendTask,
    identity: &TaskIdentity,
    outputs: &[T],
    deadline: Deadline,
) -> Result<StoredCheckpoint, RuntimeError>
where
    T: Clone + Into<StagedOutput>,
{
    deadline::ensure(deadline)?;
    let mut outputs = outputs.iter().cloned().map(Into::into).collect::<Vec<_>>();
    outputs.sort_by(|left, right| {
        output::path_string(left.target()).cmp(&output::path_string(right.target()))
    });
    let mut entries = Vec::with_capacity(outputs.len());
    for (index, value) in outputs.iter().enumerate() {
        entries.push(entry(value, task, identity, index, deadline)?);
    }
    let output_records = entries.iter().map(|value| value.record().clone()).collect();
    let payload = manifest::encode(&CheckpointManifest {
        schema_version: 2,
        outputs: entries,
    })?;
    deadline::ensure(deadline)?;
    let record = store
        .put_while(&identity.descriptor, &payload, || Instant::now() < deadline)
        .map_err(|error| super::error::artifact("cannot store render checkpoint", error))?;
    Ok(StoredCheckpoint {
        record,
        output_records,
    })
}

fn entry(
    output_value: &StagedOutput,
    task: &BackendTask,
    identity: &TaskIdentity,
    index: usize,
    deadline: Deadline,
) -> Result<CheckpointOutput, RuntimeError> {
    let path = output::path_string(output_value.target());
    let descriptor = super::identity::output(identity, task.product, &path, index);
    let key = artifact_key(&descriptor)
        .map_err(|error| super::error::artifact("cannot identify render output", error))?;
    match output_value {
        StagedOutput::File(file) => {
            let state = output::file_state_until(&file.source, file.allow_empty, deadline)?;
            safe_size(state.size_bytes)?;
            Ok(CheckpointOutput::File {
                path,
                record: ArtifactRecord {
                    key,
                    content: state.content,
                    size_bytes: state.size_bytes,
                },
                descriptor,
            })
        }
        StagedOutput::Package(package) => {
            let inventory =
                staging::inspect_package(&package.source, &package.entrypoint, deadline)?;
            if inventory != package.inventory {
                return Err(RuntimeError::new(
                    "staged package changed before checkpoint storage",
                ));
            }
            safe_size(inventory.size_bytes())?;
            Ok(CheckpointOutput::Package {
                path,
                entrypoint: output::path_string(&package.entrypoint),
                record: ArtifactRecord {
                    key,
                    content: inventory.tree.clone(),
                    size_bytes: inventory.size_bytes(),
                },
                inventory,
                descriptor,
            })
        }
    }
}

fn safe_size(size: u64) -> Result<(), RuntimeError> {
    if size <= veac_ir::MAX_SAFE_INTEGER {
        Ok(())
    } else {
        Err(RuntimeError::new(
            "render output exceeds canonical size limit",
        ))
    }
}
