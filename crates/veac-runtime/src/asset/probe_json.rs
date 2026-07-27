use std::collections::BTreeMap;

use veac_ir::{HashAlgorithm, MediaIdentity, MediaProbeSnapshot, StreamIntent};

use super::selection::select_streams;
use super::time::optional_time;
use super::ProbeError;

mod normalize;
mod raw;

pub const PROBE_SCHEMA_VERSION: u32 = veac_ir::MEDIA_PROBE_SCHEMA_VERSION;
pub const FIXTURE_PROBE_ENGINE: &str = "ffprobe version fixture";
pub const STREAM_SELECTION_POLICY: &str = "veac.default-stream.v1";
const MAX_PROBE_ENGINE_BYTES: usize = 1_024;
const MAX_PROBE_STREAMS: usize = 4_096;

/// Normalize complete ffprobe JSON using a caller-supplied, reproducible engine identifier.
pub fn parse_ffprobe_json(
    json: &str,
    observed_identity: MediaIdentity,
    intent: StreamIntent,
    engine: &str,
) -> Result<MediaProbeSnapshot, ProbeError> {
    if json.len() as u64 > veac_artifact::MAX_ARTIFACT_METADATA_BYTES {
        return Err(limit("JSON output"));
    }
    if observed_identity.algorithm != HashAlgorithm::Sha256 {
        return Err(ProbeError::UnsupportedHashAlgorithm {
            algorithm: observed_identity.algorithm,
        });
    }
    if !valid_sha256(&observed_identity.digest) {
        return Err(invalid(
            "observed_identity.digest",
            &observed_identity.digest,
        ));
    }
    if engine.len() > MAX_PROBE_ENGINE_BYTES {
        return Err(limit("engine identity"));
    }
    if engine.trim().is_empty() || engine.chars().any(char::is_control) {
        return Err(invalid("engine", engine));
    }
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(source) => return Err(ProbeError::InvalidJson { source }),
    };
    veac_artifact::validate_artifact_json(&value).map_err(|_| limit("JSON shape"))?;
    let mut data: raw::FfprobeOutput = match serde_json::from_value(value) {
        Ok(data) => data,
        Err(source) => return Err(ProbeError::InvalidJson { source }),
    };
    if data.streams.len() > MAX_PROBE_STREAMS {
        return Err(limit("stream inventory"));
    }
    data.streams.sort_by_key(|stream| stream.index);
    let mut ordinals = BTreeMap::new();
    let mut previous_index = None;
    let mut streams = Vec::with_capacity(data.streams.len());
    for stream in data.streams {
        let global_index = match stream.index {
            Some(index) => index,
            None => return Err(invalid("streams[].index", "")),
        };
        if previous_index == Some(global_index) {
            return Err(invalid("streams[].index", &global_index.to_string()));
        }
        previous_index = Some(global_index);
        let media_type = normalize::media_type(stream.codec_type.as_deref())?;
        let type_index = ordinals.entry(media_type).or_insert(0);
        streams.push(normalize::stream(
            stream,
            global_index,
            *type_index,
            media_type,
        )?);
        *type_index += 1;
    }
    let (selected_video_stream, selected_audio_stream) = select_streams(&streams, intent)?;
    let format = data
        .format
        .ok_or_else(|| invalid("format.format_name", ""))?;
    let container_format = format
        .format_name
        .filter(|value| veac_ir::container_format_valid(value))
        .ok_or_else(|| invalid("format.format_name", ""))?;
    let format_duration = format.duration;
    let container_brand = normalize_brand(format.tags.and_then(|tags| tags.major_brand))?;
    let container_duration = optional_time("format.duration", format_duration.as_deref(), true)?;
    Ok(MediaProbeSnapshot {
        schema_version: PROBE_SCHEMA_VERSION,
        engine: engine.to_owned(),
        selection_policy: STREAM_SELECTION_POLICY.to_owned(),
        container_format,
        container_brand,
        observed_identity,
        container_duration,
        streams,
        selected_video_stream,
        selected_audio_stream,
    })
}

fn valid_sha256(digest: &str) -> bool {
    if digest.len() != 64 {
        return false;
    }
    for byte in digest.bytes() {
        if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte) {
            return false;
        }
    }
    true
}

fn normalize_brand(value: Option<String>) -> Result<Option<String>, ProbeError> {
    match value {
        None => Ok(None),
        Some(ref value) if value.is_empty() || value == "N/A" => Ok(None),
        Some(value)
            if value.len() <= 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_graphic() || byte == b' ') =>
        {
            Ok(Some(value))
        }
        Some(value) => Err(invalid("format.tags.major_brand", &value)),
    }
}

fn invalid(field: &'static str, value: &str) -> ProbeError {
    ProbeError::InvalidField {
        field,
        value: value.to_owned(),
    }
}

fn limit(operation: &'static str) -> ProbeError {
    ProbeError::ResourceLimit { operation }
}
