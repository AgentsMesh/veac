use std::fmt;

use schemars::schema_for;

use super::{
    validate_source_edit_contract, SourceEditBatch, SourceEditError, MAX_SOURCE_EDIT_JSON_BYTES,
};

#[derive(Debug)]
pub enum SourceEditJsonError {
    Json(serde_json::Error),
    Invalid(SourceEditError),
}

impl fmt::Display for SourceEditJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "source-edit JSON error: {error}"),
            Self::Invalid(error) => write!(formatter, "invalid source-edit batch: {error}"),
        }
    }
}

impl std::error::Error for SourceEditJsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::Invalid(error) => Some(error),
        }
    }
}

impl From<serde_json::Error> for SourceEditJsonError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<SourceEditError> for SourceEditJsonError {
    fn from(value: SourceEditError) -> Self {
        Self::Invalid(value)
    }
}

pub fn decode_source_edit_batch_json(input: &str) -> Result<SourceEditBatch, SourceEditJsonError> {
    if input.len() > MAX_SOURCE_EDIT_JSON_BYTES {
        return Err(SourceEditError::SourceEditJsonTooLarge {
            limit: MAX_SOURCE_EDIT_JSON_BYTES,
        }
        .into());
    }
    super::strict_json::reject_duplicate_keys(input)?;
    let batch: SourceEditBatch = serde_json::from_str(input)?;
    validate_source_edit_contract(&batch)?;
    Ok(batch)
}

pub fn canonical_source_edit_batch_json(
    batch: &SourceEditBatch,
) -> Result<String, SourceEditJsonError> {
    validate_source_edit_contract(batch)?;
    serde_json_canonicalizer::to_string(batch).map_err(Into::into)
}

pub fn source_edit_batch_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(SourceEditBatch))
}
