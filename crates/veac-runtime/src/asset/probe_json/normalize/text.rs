use crate::asset::ProbeError;

const MAX_CODEC_NAME_BYTES: usize = 128;

pub(super) fn codec(value: Option<String>) -> Result<String, ProbeError> {
    required(value, "streams[].codec_name", MAX_CODEC_NAME_BYTES)
}

pub(super) fn pixel_format(value: Option<String>) -> Result<String, ProbeError> {
    required(value, "streams[].pix_fmt", 64)
}

pub(super) fn channel_layout(value: Option<String>) -> Result<String, ProbeError> {
    match value {
        None => Ok("unknown".to_owned()),
        Some(value) => required(
            Some(value),
            "streams[].channel_layout",
            veac_ir::MAX_CHANNEL_LAYOUT_BYTES,
        ),
    }
}

fn required(
    value: Option<String>,
    field: &'static str,
    max_bytes: usize,
) -> Result<String, ProbeError> {
    match value {
        Some(value)
            if !value.trim().is_empty()
                && value.len() <= max_bytes
                && !value.chars().any(char::is_control) =>
        {
            Ok(value)
        }
        _ => Err(ProbeError::InvalidField {
            field,
            value: String::new(),
        }),
    }
}
