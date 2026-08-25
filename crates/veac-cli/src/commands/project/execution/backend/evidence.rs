use std::collections::BTreeMap;
use std::path::Path;

use veac_build::{
    CancellationToken, ProducedProjectOutput, ProjectArtifactInput, ProjectBackendError,
    ProjectComputation, ProjectFileSnapshot, ProjectSourceGraphRevision,
};
use veac_runtime::executor::FfmpegEnvironment;

use super::{CliProjectBackend, ProjectResultExt};

mod bindings;
mod output;
mod source_graph;

pub(super) fn execute(
    backend: &CliProjectBackend,
    workspace: &Path,
    computation: &ProjectComputation,
    snapshot: &ProjectFileSnapshot,
    source_revision: &ProjectSourceGraphRevision,
    artifacts: &[ProjectArtifactInput],
    cancellation: &CancellationToken,
) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError> {
    cancelled(cancellation, "before evidence preparation")?;
    let authored = source_graph::prepare(
        &backend.source_root,
        &backend.packages,
        snapshot,
        source_revision,
    )?;
    let suite = veac_evidence::validate(authored.suite.clone())
        .project_context("evidence contract validation failed")?;
    let plan = veac_evidence::plan_observations(&suite)
        .project_context("evidence observation planning failed")?;
    let bound = bindings::resolve(backend, computation, artifacts, &authored.suite.sources)?;
    cancelled(cancellation, "before evidence observation")?;
    let observer = veac_runtime::observation::MediaObserver::with_tools(
        backend.ffmpeg.clone(),
        backend.ffprobe.clone(),
        veac_runtime::observation::ObservationLimits::default(),
    )
    .project_context("evidence observer preparation failed")?;
    let run = veac_evidence::execute_observations(&plan, &bound, &observer, BTreeMap::new())
        .project_context("evidence source binding failed")?;
    cancelled(cancellation, "after evidence observation")?;
    let fingerprint = FfmpegEnvironment::fingerprint(&backend.ffmpeg)
        .project_context("evidence producer fingerprint failed")?;
    let provenance = veac_evidence::build_provenance(
        &plan,
        &bound,
        fingerprint.into(),
        Some(veac_evidence::ProjectEvidenceContext {
            manifest_sha256: backend.manifest_digest.value.clone(),
            target_instance_id: computation.instance.as_str().to_owned(),
        }),
    )
    .project_context("evidence provenance failed")?;
    let bundle = veac_evidence::prepare_evidence_bundle(&suite, &plan, &run, &provenance)
        .project_context("evidence bundle preparation failed")?;
    let produced = output::publish(workspace, computation, &bundle)?;
    source_graph::verify(
        &backend.source_root,
        &backend.packages,
        snapshot,
        source_revision,
    )?;
    bindings::verify(backend, computation, artifacts, &authored.suite.sources)?;
    Ok(produced)
}

fn cancelled(cancellation: &CancellationToken, phase: &str) -> Result<(), ProjectBackendError> {
    if cancellation.is_cancelled() {
        Err(ProjectBackendError::cancelled(format!(
            "project evidence was cancelled {phase}"
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[path = "evidence/tests.rs"]
mod tests;
