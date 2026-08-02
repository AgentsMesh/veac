use veac_artifact::ArtifactStore;
use veac_codegen::emitter::{BackendAction, BackendBundle};

use super::contract;
use super::deadline::{self, BundleSetupLimits, TaskExecutionLimits};
use super::locking;
use super::model::RuntimeBundle;
use super::process::{self, FfmpegEnvironment, FfmpegFingerprint, SystemFfmpeg};
use super::snapshot;
use super::staging;
use crate::RuntimeError;

mod result;
mod run;

pub use result::{BundleExecution, TaskExecution};

pub struct BundleExecutor<E> {
    environment: E,
    setup_limits: BundleSetupLimits,
    limits: TaskExecutionLimits,
    pub(in crate::executor) checkpoint_stored_observer:
        Box<dyn Fn(std::time::Instant) -> std::time::Instant>,
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
        let parents = [contract.output_parent.clone()];
        let locks = locking::acquire_until(&parents, setup_deadline)?;
        contract::validate_filesystem_paths_until(bundle, &contract, setup_deadline)?;
        staging::recover(&locks, &parents, setup_deadline)?;
        let snapshots = snapshot::capture(bundle, setup_deadline)?;
        let fingerprint = self.preflight(bundle, setup_deadline)?;
        deadline::ensure_setup(setup_deadline)?;
        self.execute_tasks(
            bundle,
            &contract,
            store,
            &snapshots,
            fingerprint.as_ref(),
            &locks,
        )
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
