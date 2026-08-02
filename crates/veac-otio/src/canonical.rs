use schemars::schema_for;

use crate::{OtioError, OtioImportBindings, OtioLossReport, OtioTimeline};

pub fn decode_otio_json(input: &str) -> Result<OtioTimeline, OtioError> {
    veac_ir::reject_duplicate_json_keys(input)?;
    let value: OtioTimeline = serde_json::from_str(input)?;
    super::import::validate_header(&value)?;
    Ok(value)
}

pub fn canonical_otio_json(value: &OtioTimeline) -> Result<String, OtioError> {
    super::import::validate_header(value)?;
    serde_json_canonicalizer::to_string(value).map_err(Into::into)
}

pub fn canonical_otio_loss_json(value: &OtioLossReport) -> Result<String, OtioError> {
    serde_json_canonicalizer::to_string(value).map_err(Into::into)
}

pub fn otio_loss_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(OtioLossReport))
}

pub fn otio_import_bindings_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(OtioImportBindings))
}
