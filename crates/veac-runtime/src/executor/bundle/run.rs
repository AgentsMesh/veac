use std::collections::BTreeMap;
use std::path::PathBuf;

use veac_artifact::{ArtifactRecord, ArtifactStore, ContentDigest};
use veac_codegen::emitter::BackendPhase;

use super::{BundleExecution, BundleExecutor, TaskExecution};
use crate::executor::checkpoint::FreshCheckpoints;
use crate::executor::contract::ValidatedContract;
use crate::executor::locking::OutputLocks;
use crate::executor::model::RuntimeBundle;
use crate::executor::process::{FfmpegEnvironment, FfmpegFingerprint};
use crate::executor::snapshot::ResourceSnapshots;
use crate::executor::{checkpoint, contract, snapshot, staging};
use crate::RuntimeError;

struct PreviousPass {
    checkpoint: ContentDigest,
    sources: Vec<PathBuf>,
    targets: Vec<PathBuf>,
    records: Vec<ArtifactRecord>,
}

struct PendingExecution {
    result: BundleExecution,
    staged: Vec<staging::StagedTask>,
}

impl<E: FfmpegEnvironment> BundleExecutor<E> {
    pub(super) fn execute_tasks(
        &self,
        bundle: &RuntimeBundle,
        contract: &ValidatedContract,
        store: &ArtifactStore,
        snapshots: &ResourceSnapshots,
        fingerprint: Option<&FfmpegFingerprint>,
        locks: &OutputLocks,
    ) -> Result<BundleExecution, RuntimeError> {
        let mut fresh = FreshCheckpoints::new(store);
        let pending =
            match self.prepare_tasks(bundle, contract, store, snapshots, fingerprint, &mut fresh) {
                Ok(value) => value,
                Err(error) => return Err(fresh.rollback(error)),
            };
        if pending.staged.is_empty() {
            return Ok(pending.result);
        }
        let published = self.publish(bundle, contract, pending.staged, locks);
        match published {
            Ok(()) => Ok(pending.result),
            Err(failure) if failure.committed() => Err(failure.into_error()),
            Err(failure) => Err(fresh.rollback(failure.into_error())),
        }
    }

    fn prepare_tasks(
        &self,
        bundle: &RuntimeBundle,
        contract: &ValidatedContract,
        store: &ArtifactStore,
        snapshots: &ResourceSnapshots,
        fingerprint: Option<&FfmpegFingerprint>,
        fresh: &mut FreshCheckpoints<'_>,
    ) -> Result<PendingExecution, RuntimeError> {
        let mut previous = BTreeMap::<String, PreviousPass>::new();
        let mut staged_tasks = Vec::new();
        let mut executions = Vec::with_capacity(bundle.tasks.len());
        for task in &bundle.tasks {
            let deadline = self.limits.deadline()?;
            contract::verify_resources_until(bundle, &contract.resources, deadline)?;
            let predecessor = (task.phase == BackendPhase::SecondPass)
                .then(|| previous.get(&task.deliverable_id.to_string()))
                .flatten();
            let identity = checkpoint::identity(
                task,
                &contract.plan,
                &contract.resources,
                fingerprint,
                predecessor.map(|value| &value.checkpoint),
            )?;
            let (record, outputs, sources, records, cache_hit) = if let Some(hit) =
                checkpoint::resume(store, task, &identity, deadline)?
            {
                contract::verify_resources_until(bundle, &contract.resources, deadline)?;
                let sources = hit.paths.clone();
                (hit.record, hit.paths, sources, hit.output_records, true)
            } else {
                let staged = self.render_task(task, predecessor, snapshots, deadline)?;
                contract::verify_resources_until(bundle, &contract.resources, deadline)?;
                let outputs = staged.targets();
                let sources = staged.sources();
                let stored = checkpoint::store(store, task, &identity, staged.outputs(), deadline)?;
                fresh.track(identity.key.clone());
                let deadline = (self.checkpoint_stored_observer)(deadline).min(deadline);
                contract::verify_resources_until(bundle, &contract.resources, deadline)?;
                staged_tasks.push(staged);
                (
                    stored.record,
                    outputs,
                    sources,
                    stored.output_records,
                    false,
                )
            };
            if task.phase == BackendPhase::FirstPass {
                previous.insert(
                    task.deliverable_id.to_string(),
                    PreviousPass {
                        checkpoint: record.content.clone(),
                        sources,
                        targets: outputs.clone(),
                        records: records.clone(),
                    },
                );
            }
            executions.push(TaskExecution {
                deliverable_id: task.deliverable_id.clone(),
                phase: task.phase,
                cache_hit,
                outputs,
                output_records: records,
                checkpoint: record,
            });
        }
        Ok(PendingExecution {
            result: BundleExecution { tasks: executions },
            staged: staged_tasks,
        })
    }

    fn render_task(
        &self,
        task: &veac_codegen::emitter::BackendTask,
        predecessor: Option<&PreviousPass>,
        snapshots: &ResourceSnapshots,
        deadline: std::time::Instant,
    ) -> Result<staging::StagedTask, RuntimeError> {
        let executable = snapshots.rebind(task, deadline)?;
        let passlogs = predecessor
            .map(|value| {
                snapshot::ReboundPasslogs::capture(
                    &executable,
                    &value.sources,
                    &value.targets,
                    &value.records,
                    deadline,
                )
            })
            .transpose()?;
        let executable = passlogs
            .as_ref()
            .map_or(&executable, snapshot::ReboundPasslogs::task);
        staging::perform(&self.environment, executable, deadline)
    }

    fn publish(
        &self,
        bundle: &RuntimeBundle,
        contract: &ValidatedContract,
        staged: Vec<staging::StagedTask>,
        locks: &OutputLocks,
    ) -> Result<(), staging::PublicationFailure> {
        let mut publication = staging::Publication::new(&contract.output_parent)
            .map_err(staging::PublicationFailure::before_commit)?;
        for task in staged {
            publication
                .adopt(task)
                .map_err(staging::PublicationFailure::before_commit)?;
        }
        let deadline = self
            .limits
            .deadline()
            .map_err(staging::PublicationFailure::before_commit)?;
        contract::verify_resources_until(bundle, &contract.resources, deadline)
            .map_err(staging::PublicationFailure::before_commit)?;
        publication.commit(locks, deadline)
    }
}
