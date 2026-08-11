use veac_lang::program::expression::Value;

use crate::{
    InputId, MediaDerivation, ProjectOpticalFlowMethod, ProjectSegmentAudio, ProjectSourceClock,
    ProjectStreamSelection,
};

use super::literal::rational;
use super::value::{unknown_variant, Decoder, Fields};
use super::ProjectDecodeError;

pub(super) fn decode(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<MediaDerivation, ProjectDecodeError> {
    let variant = decoder.variant(value, "MediaDerivation", path)?;
    let fields = &variant.fields;
    let source = source(decoder, fields)?;
    match variant.name {
        "ProxyVideo" => Ok(MediaDerivation::ProxyVideo {
            source,
            source_stream: stream_field(decoder, fields, "source_stream")?,
            source_clock: clock_field(decoder, fields, "source_clock")?,
            width: u32_field(decoder, fields, "width")?,
            height: u32_field(decoder, fields, "height")?,
            frame_rate: rational_field(decoder, fields, "frame_rate")?,
            crf: u8_field(decoder, fields, "crf")?,
        }),
        "ProxyAudio" => Ok(MediaDerivation::ProxyAudio {
            source,
            source_stream: stream_field(decoder, fields, "source_stream")?,
            source_clock: clock_field(decoder, fields, "source_clock")?,
            sample_rate: u32_field(decoder, fields, "sample_rate")?,
            channels: u8_field(decoder, fields, "channels")?,
        }),
        "Thumbnail" => Ok(MediaDerivation::Thumbnail {
            source,
            source_stream: stream_field(decoder, fields, "source_stream")?,
            at: rational_field(decoder, fields, "at")?,
            width: u32_field(decoder, fields, "width")?,
            height: u32_field(decoder, fields, "height")?,
        }),
        "Waveform" => Ok(MediaDerivation::Waveform {
            source,
            source_stream: stream_field(decoder, fields, "source_stream")?,
            source_clock: clock_field(decoder, fields, "source_clock")?,
            sample_rate: u32_field(decoder, fields, "sample_rate")?,
            width: u32_field(decoder, fields, "width")?,
            height: u32_field(decoder, fields, "height")?,
            color: decoder.text(fields.get("color")?, &fields.path("color"))?,
        }),
        "OpticalFlow" => Ok(MediaDerivation::OpticalFlow {
            source,
            source_stream: stream_field(decoder, fields, "source_stream")?,
            source_clock: clock_field(decoder, fields, "source_clock")?,
            width: u32_field(decoder, fields, "width")?,
            height: u32_field(decoder, fields, "height")?,
            frame_rate: rational_field(decoder, fields, "frame_rate")?,
            method: method_field(decoder, fields, "method")?,
        }),
        "SourceSegment" => Ok(MediaDerivation::SourceSegment {
            source,
            video_stream: stream_field(decoder, fields, "video_stream")?,
            start: rational_field(decoder, fields, "start")?,
            duration: rational_field(decoder, fields, "duration")?,
            width: u32_field(decoder, fields, "width")?,
            height: u32_field(decoder, fields, "height")?,
            frame_rate: rational_field(decoder, fields, "frame_rate")?,
            audio: optional_audio(decoder, fields.get("audio")?, &fields.path("audio"))?,
            crf: u8_field(decoder, fields, "crf")?,
        }),
        name => Err(unknown_variant(path, "MediaDerivation", name)),
    }
}

fn source(decoder: &Decoder<'_>, fields: &Fields<'_>) -> Result<InputId, ProjectDecodeError> {
    decoder
        .identifier(fields.get("source")?, &fields.path("source"))
        .map(InputId::new)
}

fn stream_field(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
    name: &str,
) -> Result<ProjectStreamSelection, ProjectDecodeError> {
    stream(decoder, fields.get(name)?, &fields.path(name))
}

fn stream(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<ProjectStreamSelection, ProjectDecodeError> {
    let fields = decoder.structure(value, "ProjectStreamSelection", path)?;
    Ok(ProjectStreamSelection {
        global_index: u32_field(decoder, &fields, "global_index")?,
        type_index: u32_field(decoder, &fields, "type_index")?,
    })
}

fn clock_field(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
    name: &str,
) -> Result<ProjectSourceClock, ProjectDecodeError> {
    let path = fields.path(name);
    let variant = decoder.variant(fields.get(name)?, "ProjectSourceClock", &path)?;
    let duration = rational_field(decoder, &variant.fields, "duration")?;
    match variant.name {
        "Identity" => Ok(ProjectSourceClock::Identity { duration }),
        "Bounded" => Ok(ProjectSourceClock::Bounded {
            start: rational_field(decoder, &variant.fields, "start")?,
            duration,
        }),
        value => Err(unknown_variant(&path, "ProjectSourceClock", value)),
    }
}

fn method_field(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
    name: &str,
) -> Result<ProjectOpticalFlowMethod, ProjectDecodeError> {
    let path = fields.path(name);
    let variant = decoder.variant(fields.get(name)?, "ProjectOpticalFlowMethod", &path)?;
    match variant.name {
        "BlockMatching" => Ok(ProjectOpticalFlowMethod::BlockMatching),
        "MotionCompensated" => Ok(ProjectOpticalFlowMethod::MotionCompensated),
        value => Err(unknown_variant(&path, "ProjectOpticalFlowMethod", value)),
    }
}

fn optional_audio(
    decoder: &Decoder<'_>,
    value: &Value,
    path: &str,
) -> Result<Option<ProjectSegmentAudio>, ProjectDecodeError> {
    let variant = decoder.variant(value, "OptionalProjectSegmentAudio", path)?;
    match variant.name {
        "None" => Ok(None),
        "Some" => {
            let path = variant.fields.path("value");
            let fields =
                decoder.structure(variant.fields.get("value")?, "ProjectSegmentAudio", &path)?;
            Ok(Some(ProjectSegmentAudio {
                source_stream: stream_field(decoder, &fields, "source_stream")?,
                sample_rate: u32_field(decoder, &fields, "sample_rate")?,
                channels: u8_field(decoder, &fields, "channels")?,
            }))
        }
        value => Err(unknown_variant(path, "OptionalProjectSegmentAudio", value)),
    }
}

fn rational_field(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
    name: &str,
) -> Result<crate::ProjectRational, ProjectDecodeError> {
    rational(decoder, fields.get(name)?, &fields.path(name))
}

fn u32_field(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
    name: &str,
) -> Result<u32, ProjectDecodeError> {
    decoder.u32(fields.get(name)?, &fields.path(name))
}

fn u8_field(
    decoder: &Decoder<'_>,
    fields: &Fields<'_>,
    name: &str,
) -> Result<u8, ProjectDecodeError> {
    decoder.u8(fields.get(name)?, &fields.path(name))
}
