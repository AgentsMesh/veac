use std::fmt;

use schemars::schema_for;
use sha2::{Digest, Sha256};

use crate::{validate, ProjectEnvelope, ValidationErrors};

#[derive(Debug)]
pub enum CanonicalError {
    Json(serde_json::Error),
    Validation(ValidationErrors),
}

impl fmt::Display for CanonicalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "canonical JSON error: {error}"),
            Self::Validation(errors) => {
                write!(formatter, "canonical IR validation failed: {errors}")
            }
        }
    }
}

impl std::error::Error for CanonicalError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Validation(errors) => Some(errors),
        }
    }
}

impl From<serde_json::Error> for CanonicalError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn decode_canonical_json(input: &str) -> Result<ProjectEnvelope, CanonicalError> {
    crate::strict_json::reject_duplicate_keys(input)?;
    let envelope: ProjectEnvelope = serde_json::from_str(input)?;
    validate(&envelope).map_err(CanonicalError::Validation)?;
    Ok(envelope)
}

pub fn canonical_json(envelope: &ProjectEnvelope) -> Result<String, CanonicalError> {
    validate(envelope).map_err(CanonicalError::Validation)?;
    serde_json_canonicalizer::to_string(envelope).map_err(CanonicalError::Json)
}

pub fn canonical_bytes(envelope: &ProjectEnvelope) -> Result<Vec<u8>, CanonicalError> {
    validate(envelope).map_err(CanonicalError::Validation)?;
    serde_json_canonicalizer::to_vec(envelope).map_err(CanonicalError::Json)
}

/// Hash of authoring semantics. Revision, operation replay history, and probe snapshots are
/// intentionally excluded; the full canonical snapshot has a separate hash below.
pub fn semantic_hash(envelope: &ProjectEnvelope) -> Result<String, CanonicalError> {
    validate(envelope).map_err(CanonicalError::Validation)?;
    let mut semantic = envelope.clone();
    semantic.project.revision = 0;
    semantic.project.applied_operations.clear();
    for material in &mut semantic.project.materials {
        material.probe = None;
    }
    let bytes = canonical_bytes(&semantic)?;
    Ok(hex_digest(Sha256::digest(bytes)))
}

/// Hash of the full canonical snapshot, including revision and observed probe facts.
pub fn snapshot_hash(envelope: &ProjectEnvelope) -> Result<String, CanonicalError> {
    let bytes = canonical_bytes(envelope)?;
    Ok(hex_digest(Sha256::digest(bytes)))
}

pub fn project_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(ProjectEnvelope))
}

pub(crate) fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
