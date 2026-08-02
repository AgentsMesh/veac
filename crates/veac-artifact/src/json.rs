use std::io::Write;

use serde::Serialize;
use serde_json::Value;

use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, MAX_ARTIFACT_JSON_DEPTH,
    MAX_ARTIFACT_JSON_NODES, MAX_ARTIFACT_JSON_STRING_BYTES,
};

pub(crate) fn validate_shape(value: &Value) -> ArtifactResult<()> {
    let mut pending = vec![(value, 1_usize)];
    let mut nodes = 0_usize;
    let mut string_bytes = 0_usize;
    while let Some((value, depth)) = pending.pop() {
        nodes = nodes.saturating_add(1);
        if nodes > MAX_ARTIFACT_JSON_NODES || depth > MAX_ARTIFACT_JSON_DEPTH {
            return limit("artifact JSON exceeds its shape budget");
        }
        match value {
            Value::String(value) => add_string(&mut string_bytes, value.len())?,
            Value::Array(values) => {
                if nodes.saturating_add(values.len()) > MAX_ARTIFACT_JSON_NODES {
                    return limit("artifact JSON exceeds its node budget");
                }
                pending.extend(values.iter().map(|value| (value, depth + 1)));
            }
            Value::Object(values) => {
                if nodes.saturating_add(values.len()) > MAX_ARTIFACT_JSON_NODES {
                    return limit("artifact JSON exceeds its node budget");
                }
                for (key, value) in values {
                    add_string(&mut string_bytes, key.len())?;
                    pending.push((value, depth + 1));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn validate_artifact_json(value: &Value) -> ArtifactResult<()> {
    validate_shape(value)
}

pub fn canonical_artifact_json_bounded(value: &Value, max_bytes: u64) -> ArtifactResult<Vec<u8>> {
    if max_bytes == 0 || max_bytes > crate::MAX_ANALYSIS_PAYLOAD_BYTES {
        return limit("artifact JSON byte limit is outside the supported policy");
    }
    validate_shape(value)?;
    canonical_bounded(value, max_bytes, "artifact JSON cannot be canonicalized")
}

pub(crate) fn canonical_bounded<T: Serialize>(
    value: &T,
    max_bytes: u64,
    context: &str,
) -> ArtifactResult<Vec<u8>> {
    let mut output = BoundedVec::new(max_bytes);
    let result = serde_json_canonicalizer::to_writer(value, &mut output);
    if output.exceeded {
        return limit(&format!("{context} exceeds {max_bytes} bytes"));
    }
    result.map(|()| output.bytes).map_err(|error| {
        ArtifactError::with_source(ArtifactErrorKind::Serialization, context, error)
    })
}

fn add_string(total: &mut usize, bytes: usize) -> ArtifactResult<()> {
    *total = total.saturating_add(bytes);
    if *total > MAX_ARTIFACT_JSON_STRING_BYTES {
        limit("artifact JSON exceeds its string byte budget")
    } else {
        Ok(())
    }
}

struct BoundedVec {
    bytes: Vec<u8>,
    max_bytes: u64,
    exceeded: bool,
}

impl BoundedVec {
    fn new(max_bytes: u64) -> Self {
        Self {
            bytes: Vec::new(),
            max_bytes,
            exceeded: false,
        }
    }
}

impl Write for BoundedVec {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let next = (self.bytes.len() as u64).saturating_add(bytes.len() as u64);
        if next > self.max_bytes {
            self.exceeded = true;
            return Err(std::io::Error::other("artifact JSON byte limit exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn limit<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        message,
    ))
}

#[cfg(test)]
#[path = "json/tests.rs"]
mod tests;
