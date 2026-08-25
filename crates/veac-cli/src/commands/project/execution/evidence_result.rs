use veac_artifact::{ArtifactEvidenceOutcome, ArtifactParameters, ArtifactStore, ContentDigest};

use crate::error::{CliError, CliResult};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct EvidenceSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub errors: usize,
}

impl EvidenceSummary {
    pub fn is_pass(&self) -> bool {
        self.total > 0 && self.failed == 0 && self.errors == 0
    }
}

pub(super) fn inspect(
    receipt: &veac_build::ProjectBuildReceipt,
    store: &ArtifactStore,
) -> CliResult<EvidenceSummary> {
    let mut summary = EvidenceSummary::default();
    for node in receipt
        .nodes
        .iter()
        .filter(|node| node.action_kind == "evidence")
    {
        if node.outputs.len() != 1 {
            return Err(contract(
                "evidence node must publish exactly one output".to_owned(),
            ));
        }
        let outcome = outcome(store, &node.outputs[0].artifact)?;
        summary.total += 1;
        match outcome {
            ArtifactEvidenceOutcome::Pass => summary.passed += 1,
            ArtifactEvidenceOutcome::Fail => summary.failed += 1,
            ArtifactEvidenceOutcome::Error => summary.errors += 1,
        }
    }
    if summary.total == 0 {
        return Err(CliError::new(
            "PROJECT_EVIDENCE_MISSING",
            "project contains no completed evidence target",
        ));
    }
    Ok(summary)
}

fn outcome(store: &ArtifactStore, key: &ContentDigest) -> CliResult<ArtifactEvidenceOutcome> {
    let artifact = match store.open(key) {
        Ok(Some(artifact)) => artifact,
        Ok(None) => {
            return Err(contract(
                "evidence artifact is absent from the project CAS".to_owned(),
            ))
        }
        Err(error) => return Err(contract(format!("cannot open evidence artifact: {error}"))),
    };
    match &artifact.descriptor().parameters {
        ArtifactParameters::EvidenceBundle(value) => Ok(value.outcome),
        _ => Err(contract(
            "evidence node output is not a typed evidence bundle".to_owned(),
        )),
    }
}

fn contract(message: String) -> CliError {
    CliError::new("PROJECT_EVIDENCE_CONTRACT", message)
}

#[cfg(test)]
#[path = "evidence_result_tests.rs"]
mod tests;
