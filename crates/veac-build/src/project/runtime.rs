use std::path::PathBuf;

use veac_artifact::ArtifactStore;

use super::{delivery, executor::ProjectNodeExecutor};
use crate::{
    BuildErrorKind, BuildLimits, BuildResult, BuildScheduler, CancellationToken, DeliveryStatus,
    DiskBuildCache, PortName, ProjectBackend, ProjectBuildOutcome, ProjectBuildPlan,
    ProjectBuildReceipt, ProjectDeliveryReceipt,
};

pub struct ProjectBuildRuntime<B> {
    scheduler: BuildScheduler,
    cache: DiskBuildCache,
    executor: ProjectNodeExecutor<B>,
    delivery_root: PathBuf,
}

impl<B> ProjectBuildRuntime<B> {
    pub fn new(
        limits: BuildLimits,
        store: ArtifactStore,
        lease_root: impl Into<PathBuf>,
        staging_root: impl Into<PathBuf>,
        delivery_root: impl Into<PathBuf>,
        backend: B,
    ) -> BuildResult<Self> {
        Ok(Self {
            scheduler: BuildScheduler::new(limits),
            cache: DiskBuildCache::new(store.clone(), lease_root)?,
            executor: ProjectNodeExecutor::new(store, staging_root, backend)?,
            delivery_root: delivery::checked_root(delivery_root)?,
        })
    }

    pub fn artifact_store(&self) -> &ArtifactStore {
        self.executor.artifact_store()
    }
}

impl<B: ProjectBackend> ProjectBuildRuntime<B> {
    pub fn build(
        &self,
        plan: &ProjectBuildPlan,
        cancellation: CancellationToken,
    ) -> BuildResult<ProjectBuildReceipt> {
        let build = self.scheduler.run(
            &plan.graph,
            &self.executor,
            &self.cache,
            cancellation.clone(),
        )?;
        let mut receipt = ProjectBuildReceipt::capture(plan, &build)?;
        if receipt.outcome != ProjectBuildOutcome::Succeeded {
            return Ok(receipt);
        }
        let mut failed = false;
        let mut cancelled = false;
        for delivery in &plan.deliveries {
            cancelled |= cancellation.is_cancelled();
            if failed || cancelled {
                receipt.deliveries.push(ProjectDeliveryReceipt {
                    instance: delivery.instance.clone(),
                    delivery: delivery.delivery.id.clone(),
                    destination: delivery.delivery.destination.as_str().to_owned(),
                    artifact: None,
                    status: DeliveryStatus::Skipped,
                    message: Some(skip_message(cancelled).to_owned()),
                });
                continue;
            }
            let artifact = build
                .node(&delivery.node)
                .and_then(|node| node.outputs.as_ref())
                .and_then(|outputs| {
                    PortName::new(delivery.delivery.output.as_str())
                        .ok()
                        .and_then(|name| outputs.get(&name))
                })
                .cloned();
            let result = artifact
                .as_ref()
                .ok_or_else(|| {
                    crate::BuildError::cache("delivery references a missing project output")
                })
                .and_then(|artifact| {
                    delivery::publish(
                        self.executor.artifact_store(),
                        artifact,
                        &self.delivery_root,
                        delivery.delivery.destination.as_str(),
                        delivery.delivery.kind,
                        &cancellation,
                    )
                });
            let (status, message) = match result {
                Ok(_) => (DeliveryStatus::Published, None),
                Err(error) if error.kind() == BuildErrorKind::Cancelled => {
                    cancelled = true;
                    (DeliveryStatus::Skipped, Some(error.to_string()))
                }
                Err(error) => {
                    failed = true;
                    (DeliveryStatus::Failed, Some(error.to_string()))
                }
            };
            receipt.deliveries.push(ProjectDeliveryReceipt {
                instance: delivery.instance.clone(),
                delivery: delivery.delivery.id.clone(),
                destination: delivery.delivery.destination.as_str().to_owned(),
                artifact,
                status,
                message,
            });
        }
        if cancelled {
            receipt.outcome = ProjectBuildOutcome::Cancelled;
        } else if failed {
            receipt.outcome = ProjectBuildOutcome::DeliveryFailed;
        }
        Ok(receipt)
    }
}

fn skip_message(cancelled: bool) -> &'static str {
    if cancelled {
        "delivery skipped because the project build was cancelled"
    } else {
        "delivery skipped after an earlier failure"
    }
}
