use serde::{Deserialize, Serialize};
use veac_artifact::{ArtifactDescriptor, ArtifactRecord};

use crate::RuntimeError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckpointManifest {
    pub schema_version: u32,
    pub outputs: Vec<CheckpointOutput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckpointOutput {
    pub path: String,
    pub descriptor: ArtifactDescriptor,
    pub record: ArtifactRecord,
}

pub(super) fn encode(value: &CheckpointManifest) -> Result<Vec<u8>, RuntimeError> {
    serde_json_canonicalizer::to_vec(value)
        .map_err(|error| RuntimeError::new(format!("cannot encode render checkpoint: {error}")))
}

pub(super) fn decode(bytes: &[u8]) -> Result<CheckpointManifest, RuntimeError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| RuntimeError::new(format!("checkpoint is not UTF-8: {error}")))?;
    veac_ir::reject_duplicate_json_keys(text)
        .map_err(|error| RuntimeError::new(format!("checkpoint JSON is ambiguous: {error}")))?;
    let value: CheckpointManifest = serde_json::from_slice(bytes)
        .map_err(|error| RuntimeError::new(format!("checkpoint JSON is invalid: {error}")))?;
    if value.schema_version != 1 || encode(&value)? != bytes {
        return Err(RuntimeError::new(
            "checkpoint is not strict canonical schema version 1 JSON",
        ));
    }
    if value.outputs.is_empty()
        || !value
            .outputs
            .windows(2)
            .all(|pair| pair[0].path < pair[1].path)
    {
        return Err(RuntimeError::new(
            "checkpoint outputs must be non-empty, unique, and sorted",
        ));
    }
    Ok(value)
}
