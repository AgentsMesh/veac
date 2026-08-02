use veac_ir::{AudioCodec, AudioOutput, MediaProbeSnapshot, ProbedStream, RationalTime};

use super::{contract, media_error};
use crate::workflow::{media::audio_duration_matches, WorkflowResult};

pub(super) fn validate(
    probe: &MediaProbeSnapshot,
    stream: &ProbedStream,
    expected: &AudioOutput,
    expected_duration: RationalTime,
) -> WorkflowResult<()> {
    let info = stream
        .audio
        .as_ref()
        .ok_or_else(|| media_error("render-segment audio facts are missing"))?;
    if stream.codec != codec(expected.codec)
        || stream.video.is_some()
        || stream.time_base.is_none()
        || (info.sample_rate, info.channels) != (expected.sample_rate, expected.channels)
    {
        return Err(media_error(
            "render-segment audio does not match its exact delivery profile",
        ));
    }
    if stream.start_time.is_none_or(|value| value.value != 0) {
        return Err(media_error("render-segment audio does not start at zero"));
    }
    let actual = contract::duration(probe, stream)
        .ok_or_else(|| media_error("render-segment audio duration is missing"))?;
    if !audio_duration_matches(actual, expected_duration, expected.sample_rate) {
        return Err(media_error(
            "render-segment audio duration differs from its exact range",
        ));
    }
    Ok(())
}

fn codec(value: AudioCodec) -> &'static str {
    match value {
        AudioCodec::Aac => "aac",
        AudioCodec::Opus => "opus",
        AudioCodec::Flac => "flac",
        AudioCodec::PcmS16Le => "pcm_s16le",
        AudioCodec::PcmS24Le => "pcm_s24le",
        AudioCodec::PcmS32Le => "pcm_s32le",
    }
}

#[cfg(test)]
#[path = "audio/tests.rs"]
mod tests;
