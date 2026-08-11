use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ArtifactResult, ContentDigest, DigestAlgorithm};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactEvidenceOutcome {
    Pass,
    Fail,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBundleParameters {
    pub suite: ContentDigest,
    pub report: ContentDigest,
    pub outcome: ArtifactEvidenceOutcome,
}

impl EvidenceBundleParameters {
    pub fn new(
        suite_sha256: impl Into<String>,
        report_sha256: impl Into<String>,
        outcome: ArtifactEvidenceOutcome,
    ) -> Self {
        Self {
            suite: sha256(suite_sha256),
            report: sha256(report_sha256),
            outcome,
        }
    }

    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        self.suite.validate()?;
        self.report.validate()
    }
}

fn sha256(value: impl Into<String>) -> ContentDigest {
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: value.into(),
    }
}
