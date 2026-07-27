use std::collections::BTreeMap;
use std::path::PathBuf;

use veac_artifact::{ArtifactRecord, ArtifactStore, ContentDigest};
use veac_codegen::emitter::{BackendAction, BackendBundle, BackendPhase};

use super::checkpoint;
use super::contract;
use super::deadline::{self, BundleSetupLimits, TaskExecutionLimits};
use super::locking;
use super::model::RuntimeBundle;
use super::process::{self, FfmpegEnvironment, FfmpegFingerprint, SystemFfmpeg};
use super::snapshot;
use super::staging;
use crate::RuntimeError;

mod result;

pub use result::{BundleExecution, TaskExecution};

pub struct BundleExecutor<E> {
    environment: E,
    setup_limits: BundleSetupLimits,
    limits: TaskExecutionLimits,
    pub(in crate::executor) checkpoint_stored_observer:
        Box<dyn Fn(std::time::Instant) -> std::time::Instant>,
}

struct PreviousPass {
    checkpoint: ContentDigest,
    outputs: Vec<PathBuf>,
    output_records: Vec<ArtifactRecord>,
}

impl<E> BundleExecutor<E> {
    pub fn new(environment: E) -> Self {
        Self {
            environment,
            setup_limits: BundleSetupLimits::default(),
            limits: TaskExecutionLimits::default(),
            checkpoint_stored_observer: Box::new(|deadline| deadline),
        }
    }

    pub fn environment(&self) -> &E {
        &self.environment
    }

    pub fn with_limits(mut self, limits: TaskExecutionLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_setup_limits(mut self, limits: BundleSetupLimits) -> Self {
        self.setup_limits = limits;
        self
    }
}

impl<E: FfmpegEnvironment> BundleExecutor<E> {
    pub fn execute(
        &self,
        bundle: &BackendBundle,
        store: &ArtifactStore,
    ) -> Result<BundleExecution, RuntimeError> {
        let bundle = RuntimeBundle::from_backend(bundle);
        self.execute_runtime(&bundle, store)
    }

    pub(in crate::executor) fn execute_runtime(
        &self,
        bundle: &RuntimeBundle,
        store: &ArtifactStore,
    ) -> Result<BundleExecution, RuntimeError> {
        let setup_deadline = self.setup_limits.deadline()?;
        let contract = contract::validate_until(bundle, setup_deadline)?;
        contract::validate_store(bundle, store.root())?;
        deadline::ensure_setup(setup_deadline)?;
        let _locks = locking::acquire_until(&contract.output_parents, setup_deadline)?;
        contract::validate_filesystem_paths_until(bundle, &contract, setup_deadline)?;
        staging::recover(&_locks, &contract.output_parents, setup_deadline)?;
        let snapshots = snapshot::capture(bundle, setup_deadline)?;
        let fingerprint = self.preflight(bundle, setup_deadline)?;
        deadline::ensure_setup(setup_deadline)?;
        let mut previous = BTreeMap::<String, PreviousPass>::new();
        let mut executions = Vec::with_capacity(bundle.tasks.len());
        for task in &bundle.tasks {
            let deadline = self.limits.deadline()?;
            contract::verify_resources_until(bundle, &contract.resources, deadline)?;
            let predecessor = match task.phase {
                BackendPhase::SecondPass => previous.get(&task.deliverable_id.to_string()),
                _ => None,
            };
            let identity = checkpoint::identity(
                task,
                &contract.plan,
                &contract.resources,
                fingerprint.as_ref(),
                predecessor.map(|value| &value.checkpoint),
            )?;
            let (record, outputs, output_records, cache_hit) = if let Some(hit) =
                checkpoint::resume(store, task, &identity, deadline)?
            {
                contract::verify_resources_until(bundle, &contract.resources, deadline)?;
                (hit.record, hit.paths, hit.output_records, true)
            } else {
                let executable = snapshots.rebind(task, deadline)?;
                let passlogs = predecessor
                    .map(|value| {
                        snapshot::ReboundPasslogs::capture(
                            &executable,
                            &value.outputs,
                            &value.output_records,
                            deadline,
                        )
                    })
                    .transpose()?;
                let executable = passlogs
                    .as_ref()
                    .map_or(&executable, snapshot::ReboundPasslogs::task);
                let staged = staging::perform(&self.environment, executable, deadline)?;
                contract::verify_resources_until(bundle, &contract.resources, deadline)?;
                let outputs = staged.targets();
                let stored = checkpoint::store(store, task, &identity, staged.files(), deadline)?;
                let deadline = (self.checkpoint_stored_observer)(deadline).min(deadline);
                if let Err(error) =
                    contract::verify_resources_until(bundle, &contract.resources, deadline)
                {
                    return Err(checkpoint::after_failure(store, &identity.key, error));
                }
                if let Err(error) = staged.commit(&_locks, deadline) {
                    return Err(checkpoint::after_failure(store, &identity.key, error));
                }
                (stored.record, outputs, stored.output_records, false)
            };
            if task.phase == BackendPhase::FirstPass {
                previous.insert(
                    task.deliverable_id.to_string(),
                    PreviousPass {
                        checkpoint: record.content.clone(),
                        outputs: outputs.clone(),
                        output_records: output_records.clone(),
                    },
                );
            }
            executions.push(TaskExecution {
                deliverable_id: task.deliverable_id.clone(),
                phase: task.phase,
                cache_hit,
                outputs,
                output_records,
                checkpoint: record,
            });
        }
        Ok(BundleExecution { tasks: executions })
    }

    fn preflight(
        &self,
        bundle: &RuntimeBundle,
        deadline: std::time::Instant,
    ) -> Result<Option<FfmpegFingerprint>, RuntimeError> {
        let uses_ffmpeg = bundle
            .tasks
            .iter()
            .any(|task| matches!(task.action, BackendAction::Ffmpeg(_)));
        let fingerprint = uses_ffmpeg
            .then(|| self.environment.fingerprint_until(deadline))
            .transpose()?;
        process::validate_requirements_until(&self.environment, &bundle.requirements, deadline)?;
        Ok(fingerprint)
    }
}

/// Execute a sealed codegen bundle using its embedded render-plan identity.
///
/// A caller cannot substitute an unrelated plan hash at execution time:
///
/// ```compile_fail
/// use veac_artifact::ArtifactStore;
/// use veac_codegen::emitter::BackendBundle;
/// use veac_runtime::executor::execute_bundle;
///
/// fn forged(bundle: &BackendBundle, store: &ArtifactStore) {
///     execute_bundle(bundle, "external-plan-hash", store).unwrap();
/// }
/// ```
pub fn execute_bundle(
    bundle: &BackendBundle,
    store: &ArtifactStore,
) -> Result<BundleExecution, RuntimeError> {
    BundleExecutor::new(SystemFfmpeg::default()).execute(bundle, store)
}
