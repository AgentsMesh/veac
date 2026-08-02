use std::fmt;

use schemars::schema_for;
use sha2::{Digest, Sha256};

use crate::ResolvedRenderPlan;

#[derive(Debug)]
pub struct PlanSerializationError(serde_json::Error);

impl fmt::Display for PlanSerializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "render plan serialization failed: {}", self.0)
    }
}

impl std::error::Error for PlanSerializationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl From<serde_json::Error> for PlanSerializationError {
    fn from(value: serde_json::Error) -> Self {
        Self(value)
    }
}

pub fn canonical_plan_json(plan: &ResolvedRenderPlan) -> Result<String, PlanSerializationError> {
    validate_json_round_trip(plan)?;
    serde_json_canonicalizer::to_string(plan).map_err(Into::into)
}

pub fn canonical_plan_bytes(plan: &ResolvedRenderPlan) -> Result<Vec<u8>, PlanSerializationError> {
    validate_json_round_trip(plan)?;
    serde_json_canonicalizer::to_vec(plan).map_err(Into::into)
}

pub fn plan_hash(plan: &ResolvedRenderPlan) -> Result<String, PlanSerializationError> {
    let digest = Sha256::digest(canonical_plan_bytes(plan)?);
    Ok(hex_digest(digest))
}

pub fn render_plan_json_schema() -> Result<serde_json::Value, PlanSerializationError> {
    serde_json::to_value(schema_for!(ResolvedRenderPlan)).map_err(Into::into)
}

fn validate_json_round_trip(plan: &ResolvedRenderPlan) -> Result<(), PlanSerializationError> {
    let bytes = serde_json::to_vec(plan)?;
    serde_json::from_slice::<ResolvedRenderPlan>(&bytes)?;
    Ok(())
}

fn hex_digest(bytes: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = bytes.as_ref();
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
