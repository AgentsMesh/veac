use veac_ir::{Rational, VideoCadence, VideoStreamInfo};

use super::{invalid, raw, text};
use crate::asset::time::{optional_positive_ratio, sample_aspect_ratio};
use crate::asset::ProbeError;

pub(super) fn info(
    stream: &raw::FfprobeStream,
    auxiliary: bool,
) -> Result<VideoStreamInfo, ProbeError> {
    Ok(VideoStreamInfo {
        width: positive(stream.width, "streams[].width")?,
        height: positive(stream.height, "streams[].height")?,
        frame_rate: frame_rate(stream, auxiliary)?,
        cadence: VideoCadence::Unknown,
        pixel_format: text::pixel_format(stream.pix_fmt.clone())?,
        profile: optional_text(stream.profile.as_deref(), "streams[].profile")?,
        level: level(stream.level)?,
        sample_aspect_ratio: sample_aspect_ratio(stream.sample_aspect_ratio.as_deref())?,
        rotation_degrees: rotation(stream)?,
    })
}

fn frame_rate(
    stream: &raw::FfprobeStream,
    auxiliary: bool,
) -> Result<Option<Rational>, ProbeError> {
    match parsed_rate(stream.avg_frame_rate.as_deref(), auxiliary)? {
        Some(value) => Ok(Some(value)),
        None => parsed_rate(stream.r_frame_rate.as_deref(), auxiliary),
    }
}

fn parsed_rate(raw: Option<&str>, auxiliary: bool) -> Result<Option<Rational>, ProbeError> {
    let Some(value) = optional_positive_ratio("streams[].frame_rate", raw)? else {
        return Ok(None);
    };
    let within_limit = i128::from(value.numerator)
        <= i128::from(veac_ir::MAX_FRAME_RATE) * i128::from(value.denominator);
    match (within_limit, auxiliary) {
        (true, _) => Ok(Some(value)),
        (false, true) => Ok(None),
        (false, false) => Err(invalid(
            "streams[].frame_rate",
            &format!("{}/{}", value.numerator, value.denominator),
        )),
    }
}

fn rotation(stream: &raw::FfprobeStream) -> Result<i16, ProbeError> {
    let mut raw = None;
    for data in &stream.side_data_list {
        if let Some(rotation) = &data.rotation {
            raw = Some(rotation.to_string());
            break;
        }
    }
    if raw.is_none() {
        if let Some(tags) = &stream.tags {
            raw = tags.rotate.clone();
        }
    }
    let Some(raw) = raw else {
        return Ok(0);
    };
    let parsed = match raw.trim_matches('"').parse::<f64>() {
        Ok(value) if value.is_finite() && value.fract() == 0.0 => value,
        _ => return Err(invalid("streams[].rotation", &raw)),
    };
    i16::try_from(parsed as i64).map_err(|_| invalid("streams[].rotation", &raw))
}

fn optional_text(value: Option<&str>, field: &'static str) -> Result<Option<String>, ProbeError> {
    match value {
        None | Some("") | Some("N/A") | Some("unknown") => Ok(None),
        Some(value) if value.len() > 128 || value.chars().any(char::is_control) => {
            Err(invalid(field, value))
        }
        Some(value) => Ok(Some(value.to_owned())),
    }
}

fn level(value: Option<i32>) -> Result<Option<i32>, ProbeError> {
    match value {
        None | Some(-99) => Ok(None),
        Some(value) if value >= 0 => Ok(Some(value)),
        Some(value) => Err(invalid("streams[].level", &value.to_string())),
    }
}

fn positive(value: Option<u32>, field: &'static str) -> Result<u32, ProbeError> {
    match value {
        Some(value) if value > 0 => Ok(value),
        _ => Err(invalid(field, "")),
    }
}
