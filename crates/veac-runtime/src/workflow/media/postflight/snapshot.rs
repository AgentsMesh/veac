use veac_artifact::MediaArtifactSpec;
use veac_ir::{MediaProbeSnapshot, ProbedStream, ProbedStreamType, Rational, RationalTime};

use super::super::{WorkflowError, WorkflowErrorKind, WorkflowResult};
use super::{intent, timing};

pub(super) fn validate(probe: &MediaProbeSnapshot, spec: &MediaArtifactSpec) -> WorkflowResult<()> {
    intent::expected(spec).validate(probe)?;
    match spec {
        MediaArtifactSpec::ProxyVideo(value) => video(
            probe,
            value.width,
            value.height,
            "h264",
            Some(value.frame_rate),
            Some(value.source_clock.duration()),
        ),
        MediaArtifactSpec::ProxyAudio(value) => audio(
            probe,
            value.sample_rate,
            value.channels,
            "pcm_s16le",
            Some(value.source_clock.duration()),
        ),
        MediaArtifactSpec::Waveform(value) => {
            video(probe, value.width, value.height, "png", None, None)
        }
        MediaArtifactSpec::Thumbnail(value) => {
            video(probe, value.width, value.height, "png", None, None)
        }
        MediaArtifactSpec::OpticalFlow(value) => video(
            probe,
            value.width,
            value.height,
            "h264",
            Some(value.frame_rate),
            Some(value.source_clock.duration()),
        ),
        MediaArtifactSpec::SourceSegment(value) => {
            video(
                probe,
                value.width,
                value.height,
                "h264",
                Some(value.frame_rate),
                Some(value.duration),
            )?;
            if let Some(settings) = value.audio {
                audio(
                    probe,
                    settings.sample_rate,
                    settings.channels,
                    "aac",
                    Some(value.duration),
                )?;
            }
            Ok(())
        }
    }
}

fn video(
    probe: &MediaProbeSnapshot,
    width: u32,
    height: u32,
    codec: &str,
    frame_rate: Option<Rational>,
    duration: Option<RationalTime>,
) -> WorkflowResult<()> {
    let stream = selected(probe, ProbedStreamType::Video)?;
    let facts = stream
        .video
        .as_ref()
        .ok_or_else(|| output_error("derived artifact has no video facts"))?;
    if stream.codec != codec
        || (facts.width, facts.height) != (width, height)
        || frame_rate.is_some_and(|value| facts.frame_rate != Some(value))
        || stream.time_base.is_none()
    {
        return invalid("derived video does not match its artifact contract");
    }
    zero_origin(stream, duration.is_none())?;
    if let Some(expected) = duration {
        let rate = frame_rate.ok_or_else(|| output_error("derived video has no frame rate"))?;
        complete(probe, stream, expected, DurationGranularity::Video(rate))?;
    }
    Ok(())
}

fn audio(
    probe: &MediaProbeSnapshot,
    sample_rate: u32,
    channels: u8,
    codec: &str,
    duration: Option<RationalTime>,
) -> WorkflowResult<()> {
    let stream = selected(probe, ProbedStreamType::Audio)?;
    let facts = stream
        .audio
        .as_ref()
        .ok_or_else(|| output_error("derived artifact has no audio facts"))?;
    if stream.codec != codec
        || (facts.sample_rate, facts.channels) != (sample_rate, channels)
        || stream.time_base.is_none()
    {
        return invalid("derived audio does not match its artifact contract");
    }
    zero_origin(stream, codec == "pcm_s16le")?;
    if let Some(expected) = duration {
        complete(
            probe,
            stream,
            expected,
            DurationGranularity::Audio(sample_rate),
        )?;
    }
    Ok(())
}

fn selected(probe: &MediaProbeSnapshot, kind: ProbedStreamType) -> WorkflowResult<&ProbedStream> {
    let selection = match kind {
        ProbedStreamType::Video => probe.selected_video_stream,
        ProbedStreamType::Audio => probe.selected_audio_stream,
        _ => None,
    }
    .ok_or_else(|| output_error("derived artifact has no selected media stream"))?;
    probe
        .streams
        .iter()
        .find(|stream| {
            stream.global_index == selection.global_index
                && stream.type_index == selection.type_index
                && stream.media_type == kind
        })
        .ok_or_else(|| output_error("derived artifact selection is inconsistent"))
}

pub(super) fn zero_origin(stream: &ProbedStream, allow_missing: bool) -> WorkflowResult<()> {
    match stream.start_time {
        Some(value) if value.value == 0 => Ok(()),
        None if allow_missing => Ok(()),
        _ => invalid("derived artifact stream does not start at zero"),
    }
}

enum DurationGranularity {
    Video(Rational),
    Audio(u32),
}

fn complete(
    probe: &MediaProbeSnapshot,
    stream: &ProbedStream,
    expected: RationalTime,
    granularity: DurationGranularity,
) -> WorkflowResult<()> {
    let actual = stream.duration.or(probe.container_duration);
    let valid = actual.is_some_and(|value| match granularity {
        DurationGranularity::Video(rate) => timing::video(value, expected, rate),
        DurationGranularity::Audio(sample_rate) => timing::audio(value, expected, sample_rate),
    });
    if !valid {
        return invalid("derived artifact duration does not match its source clock");
    }
    Ok(())
}

fn invalid<T>(message: &str) -> WorkflowResult<T> {
    Err(output_error(message))
}

fn output_error(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::ToolFailure, message)
}

#[cfg(test)]
#[path = "snapshot/tests.rs"]
mod tests;
