use veac_ir::{AudioStreamInfo, ProbedStream, ProbedStreamType, StreamDisposition};

use super::raw;
use crate::asset::time::{optional_positive_ratio, optional_time};
use crate::asset::ProbeError;

mod text;
mod video;

pub(super) fn stream(
    stream: raw::FfprobeStream,
    global_index: u32,
    type_index: u32,
    media_type: ProbedStreamType,
) -> Result<ProbedStream, ProbeError> {
    let time_base = optional_positive_ratio("streams[].time_base", stream.time_base.as_deref())?;
    let codec = text::codec(stream.codec_name.clone())?;
    let disposition = StreamDisposition {
        default: flag(
            "streams[].disposition.default",
            stream.disposition.is_default,
        )?,
        attached_picture: flag(
            "streams[].disposition.attached_pic",
            stream.disposition.attached_pic,
        )?,
        timed_thumbnail: flag(
            "streams[].disposition.timed_thumbnails",
            stream.disposition.timed_thumbnails,
        )?,
    };
    if media_type != ProbedStreamType::Video
        && (disposition.attached_picture || disposition.timed_thumbnail)
    {
        return Err(invalid("streams[].disposition", &global_index.to_string()));
    }
    let video = if media_type == ProbedStreamType::Video {
        Some(video::info(
            &stream,
            disposition.attached_picture || disposition.timed_thumbnail,
        )?)
    } else {
        None
    };
    let audio = if media_type == ProbedStreamType::Audio {
        Some(audio_info(&stream)?)
    } else {
        None
    };
    let playable_video = media_type == ProbedStreamType::Video
        && !disposition.attached_picture
        && !disposition.timed_thumbnail;
    if (playable_video
        && (time_base.is_none()
            || video
                .as_ref()
                .is_some_and(|value| value.frame_rate.is_none())))
        || (media_type == ProbedStreamType::Audio && time_base.is_none())
    {
        return Err(invalid("streams[].timing", &global_index.to_string()));
    }
    Ok(ProbedStream {
        global_index,
        type_index,
        media_type,
        codec,
        time_base,
        start_time: optional_time("streams[].start_time", stream.start_time.as_deref(), false)?,
        duration: optional_time("streams[].duration", stream.duration.as_deref(), true)?,
        disposition,
        video,
        audio,
    })
}

pub(super) fn media_type(raw: Option<&str>) -> Result<ProbedStreamType, ProbeError> {
    match raw {
        Some("video") => Ok(ProbedStreamType::Video),
        Some("audio") => Ok(ProbedStreamType::Audio),
        Some("subtitle") => Ok(ProbedStreamType::Subtitle),
        Some("data") => Ok(ProbedStreamType::Data),
        Some("attachment") => Ok(ProbedStreamType::Attachment),
        Some(value) => Err(invalid("streams[].codec_type", value)),
        None => Err(invalid("streams[].codec_type", "")),
    }
}

fn audio_info(stream: &raw::FfprobeStream) -> Result<AudioStreamInfo, ProbeError> {
    let raw_rate = stream.sample_rate.as_deref().unwrap_or("");
    let sample_rate = match raw_rate.parse::<u32>() {
        Ok(value) if value > 0 => value,
        _ => return Err(invalid("streams[].sample_rate", raw_rate)),
    };
    let channels = match stream.channels {
        Some(raw) => match u8::try_from(raw) {
            Ok(value) if value > 0 => value,
            _ => {
                return Err(invalid("streams[].channels", &raw.to_string()));
            }
        },
        None => return Err(invalid("streams[].channels", "")),
    };
    Ok(AudioStreamInfo {
        sample_rate,
        channels,
        channel_layout: text::channel_layout(stream.channel_layout.clone())?,
    })
}

fn flag(field: &'static str, value: u8) -> Result<bool, ProbeError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(invalid(field, &value.to_string())),
    }
}

fn invalid(field: &'static str, value: &str) -> ProbeError {
    ProbeError::InvalidField {
        field,
        value: value.to_owned(),
    }
}
