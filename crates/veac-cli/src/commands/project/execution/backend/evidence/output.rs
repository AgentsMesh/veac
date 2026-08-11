use std::path::{Path, PathBuf};

use veac_build::{ProducedProjectOutput, ProjectBackendError, ProjectComputation};
use veac_project::ProjectOutput;

use super::super::{failed, ProjectResultExt};

pub(super) fn publish(
    workspace: &Path,
    computation: &ProjectComputation,
    bundle: &veac_evidence::PreparedEvidenceBundle,
) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError> {
    let [ProjectOutput::Directory { id }] = computation.outputs.as_slice() else {
        return Err(failed(
            "an evidence target requires exactly one directory output",
        ));
    };
    let relative = PathBuf::from("outputs").join(id.as_str());
    veac_evidence::publish_evidence_bundle(bundle, &workspace.join(&relative))
        .project_context("evidence bundle publication failed")?;
    Ok(vec![ProducedProjectOutput {
        output: id.clone(),
        relative_path: relative,
        semantics: veac_build::ProjectOutputSemantics::Evidence {
            suite_sha256: bundle.manifest.suite_sha256.clone(),
            report_sha256: bundle.manifest.report_sha256.clone(),
            outcome: outcome(bundle.manifest.outcome),
        },
    }])
}

fn outcome(value: veac_evidence::EvidenceOutcome) -> veac_artifact::ArtifactEvidenceOutcome {
    match value {
        veac_evidence::EvidenceOutcome::Pass => veac_artifact::ArtifactEvidenceOutcome::Pass,
        veac_evidence::EvidenceOutcome::Fail => veac_artifact::ArtifactEvidenceOutcome::Fail,
        veac_evidence::EvidenceOutcome::Error => veac_artifact::ArtifactEvidenceOutcome::Error,
    }
}

#[cfg(test)]
#[path = "output/tests.rs"]
mod tests;
