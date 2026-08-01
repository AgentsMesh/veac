use serde::{Deserialize, Serialize};
use veac_artifact::{
    ArtifactDescriptor, ArtifactRecord, DeliveryPackageInventory, MAX_ARTIFACT_METADATA_BYTES,
};

use crate::RuntimeError;

const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CheckpointManifest {
    pub schema_version: u32,
    pub outputs: Vec<CheckpointOutput>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum CheckpointOutput {
    File {
        path: String,
        descriptor: ArtifactDescriptor,
        record: ArtifactRecord,
    },
    Package {
        path: String,
        entrypoint: String,
        inventory: DeliveryPackageInventory,
        descriptor: ArtifactDescriptor,
        record: ArtifactRecord,
    },
}

impl CheckpointOutput {
    pub fn path(&self) -> &str {
        match self {
            Self::File { path, .. } | Self::Package { path, .. } => path,
        }
    }

    pub fn descriptor(&self) -> &ArtifactDescriptor {
        match self {
            Self::File { descriptor, .. } | Self::Package { descriptor, .. } => descriptor,
        }
    }

    pub fn record(&self) -> &ArtifactRecord {
        match self {
            Self::File { record, .. } | Self::Package { record, .. } => record,
        }
    }

    pub fn into_record(self) -> ArtifactRecord {
        match self {
            Self::File { record, .. } | Self::Package { record, .. } => record,
        }
    }
}

pub(super) fn encode(value: &CheckpointManifest) -> Result<Vec<u8>, RuntimeError> {
    let bytes = serde_json_canonicalizer::to_vec(value)
        .map_err(|error| RuntimeError::new(format!("cannot encode render checkpoint: {error}")))?;
    if bytes.len() as u64 > MAX_ARTIFACT_METADATA_BYTES {
        return Err(RuntimeError::resource_limit(
            "render checkpoint manifest exceeds its metadata budget",
        ));
    }
    Ok(bytes)
}

pub(super) fn decode(bytes: &[u8]) -> Result<CheckpointManifest, RuntimeError> {
    if bytes.len() as u64 > MAX_ARTIFACT_METADATA_BYTES {
        return Err(RuntimeError::resource_limit(
            "render checkpoint manifest exceeds its metadata budget",
        ));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|error| RuntimeError::new(format!("checkpoint is not UTF-8: {error}")))?;
    veac_ir::reject_duplicate_json_keys(text)
        .map_err(|error| RuntimeError::new(format!("checkpoint JSON is ambiguous: {error}")))?;
    let value: CheckpointManifest = serde_json::from_slice(bytes)
        .map_err(|error| RuntimeError::new(format!("checkpoint JSON is invalid: {error}")))?;
    if value.schema_version != SCHEMA_VERSION || encode(&value)? != bytes {
        return Err(RuntimeError::new(
            "checkpoint is not strict canonical schema version 2 JSON",
        ));
    }
    if value.outputs.is_empty()
        || !value
            .outputs
            .windows(2)
            .all(|pair| pair[0].path() < pair[1].path())
    {
        return Err(RuntimeError::new(
            "checkpoint outputs must be non-empty, unique, and sorted",
        ));
    }
    Ok(value)
}
