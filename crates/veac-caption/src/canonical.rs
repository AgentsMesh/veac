use schemars::schema_for;
use sha2::{Digest, Sha256};

use crate::{
    validate, CaptionDocumentBindings, CaptionEnvelope, CaptionError,
    CaptionTrackInsertionBindings, LossReport,
};

pub fn decode_caption_json(input: &str) -> Result<CaptionEnvelope, CaptionError> {
    veac_ir::reject_duplicate_json_keys(input)?;
    let value: CaptionEnvelope = serde_json::from_str(input)?;
    validate(&value)?;
    Ok(value)
}

pub fn canonical_caption_json(value: &CaptionEnvelope) -> Result<String, CaptionError> {
    validate(value)?;
    serde_json_canonicalizer::to_string(value).map_err(CaptionError::Json)
}

pub fn canonical_caption_bytes(value: &CaptionEnvelope) -> Result<Vec<u8>, CaptionError> {
    validate(value)?;
    serde_json_canonicalizer::to_vec(value).map_err(CaptionError::Json)
}

pub fn caption_hash(value: &CaptionEnvelope) -> Result<String, CaptionError> {
    let digest = Sha256::digest(canonical_caption_bytes(value)?);
    Ok(hex(digest))
}

pub fn canonical_loss_report_json(value: &LossReport) -> Result<String, CaptionError> {
    serde_json_canonicalizer::to_string(value).map_err(CaptionError::Json)
}

pub fn caption_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(CaptionEnvelope))
}

pub fn caption_track_insertion_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(CaptionTrackInsertionBindings))
}

pub fn caption_document_bindings_json_schema() -> Result<serde_json::Value, serde_json::Error> {
    serde_json::to_value(schema_for!(CaptionDocumentBindings))
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
