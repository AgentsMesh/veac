use std::fmt;

use schemars::schema_for;

use crate::{EditBatch, EditOutcome};

#[derive(Debug)]
pub enum EditJsonError {
    Json(serde_json::Error),
    Invalid(&'static str),
}

impl fmt::Display for EditJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "edit JSON error: {error}"),
            Self::Invalid(message) => write!(formatter, "invalid edit batch: {message}"),
        }
    }
}

impl std::error::Error for EditJsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<serde_json::Error> for EditJsonError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

pub fn decode_edit_batch_json(input: &str) -> Result<EditBatch, EditJsonError> {
    crate::strict_json::reject_duplicate_keys(input)?;
    let batch: EditBatch = serde_json::from_str(input)?;
    validate_contract(&batch)?;
    Ok(batch)
}

pub fn canonical_edit_batch_json(batch: &EditBatch) -> Result<String, EditJsonError> {
    validate_contract(batch)?;
    serde_json_canonicalizer::to_string(batch).map_err(Into::into)
}

pub fn canonical_edit_outcome_json(outcome: &EditOutcome) -> Result<String, EditJsonError> {
    serde_json_canonicalizer::to_string(outcome).map_err(Into::into)
}

pub fn edit_batch_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(EditBatch))
}

pub fn edit_outcome_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(EditOutcome))
}

fn validate_contract(batch: &EditBatch) -> Result<(), EditJsonError> {
    let value = serde_json::to_value(batch)?;
    if !batch.atomic {
        return Err(EditJsonError::Invalid("atomic must be true"));
    }
    if batch.operations.is_empty() {
        return Err(EditJsonError::Invalid("operations must be non-empty"));
    }
    if !batch.operation_id.is_valid() || !crate::time::safe_u64(batch.base_revision) {
        return Err(EditJsonError::Invalid(
            "operation ID or base revision is invalid",
        ));
    }
    if !crate::validation::json_value_is_ijson(&value) {
        return Err(EditJsonError::Invalid(
            "values must use the exact I-JSON domain",
        ));
    }
    Ok(())
}
