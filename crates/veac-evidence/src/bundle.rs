use std::collections::BTreeSet;
use std::path::{Component, Path};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    evidence_cache_key, BundleArtifactContent, BundleArtifactRecord, EvidenceBundleManifestV1,
    EvidenceProvenanceV1, EvidenceReportV1, EvidenceSuiteV1, EVIDENCE_BUNDLE_SCHEMA_VERSION,
};

mod derivative;
mod prepare;
mod publish;

pub use prepare::prepare_evidence_bundle;
pub use publish::publish_evidence_bundle;

#[derive(Debug)]
pub enum BundleError {
    Encode(String),
    UnsafePath(String),
    DuplicatePath(String),
    NonFiniteMetric(String),
    InvalidContract(String),
    Io(std::io::Error),
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Encode(value) => write!(formatter, "cannot encode canonical JSON: {value}"),
            Self::UnsafePath(value) => write!(formatter, "unsafe bundle path: {value}"),
            Self::DuplicatePath(value) => write!(formatter, "duplicate bundle path: {value}"),
            Self::NonFiniteMetric(value) => write!(formatter, "non-finite report metric: {value}"),
            Self::InvalidContract(value) => write!(formatter, "invalid evidence contract: {value}"),
            Self::Io(error) => write!(formatter, "evidence bundle I/O failed: {error}"),
        }
    }
}

impl std::error::Error for BundleError {}

impl From<std::io::Error> for BundleError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn canonical_json(value: &impl Serialize) -> Result<Vec<u8>, BundleError> {
    serde_json_canonicalizer::to_vec(value).map_err(|error| BundleError::Encode(error.to_string()))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn build_bundle_manifest(
    suite: &EvidenceSuiteV1,
    report: &EvidenceReportV1,
    provenance: &EvidenceProvenanceV1,
    mut artifacts: Vec<BundleArtifactContent>,
) -> Result<EvidenceBundleManifestV1, BundleError> {
    finite_report(report)?;
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));
    let mut paths = BTreeSet::new();
    let mut records = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        safe_path(&artifact.path)?;
        if !paths.insert(artifact.path.clone()) {
            return Err(BundleError::DuplicatePath(artifact.path));
        }
        records.push(BundleArtifactRecord {
            path: artifact.path,
            size_bytes: artifact.bytes.len() as u64,
            sha256: sha256_hex(&artifact.bytes),
        });
    }
    let suite_sha256 = sha256_hex(&canonical_json(suite)?);
    if provenance.suite_sha256 != suite_sha256 {
        return Err(BundleError::InvalidContract(
            "provenance does not identify this suite".into(),
        ));
    }
    Ok(EvidenceBundleManifestV1 {
        schema_version: EVIDENCE_BUNDLE_SCHEMA_VERSION,
        suite_sha256,
        provenance_sha256: sha256_hex(&canonical_json(provenance)?),
        report_sha256: sha256_hex(&canonical_json(report)?),
        cache_key: evidence_cache_key(provenance)?,
        outcome: report.outcome,
        artifacts: records,
    })
}

fn finite_report(report: &EvidenceReportV1) -> Result<(), BundleError> {
    for assertion in &report.assertions {
        let values = assertion
            .metrics
            .values()
            .copied()
            .chain(assertion.series.values().flatten().copied());
        if values.into_iter().any(|value| !value.is_finite()) {
            return Err(BundleError::NonFiniteMetric(assertion.id.clone()));
        }
    }
    Ok(())
}

fn safe_path(value: &str) -> Result<(), BundleError> {
    let path = Path::new(value);
    let valid = !value.is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)));
    if valid {
        Ok(())
    } else {
        Err(BundleError::UnsafePath(value.to_owned()))
    }
}
