use veac_artifact::MediaArtifactLimits;
use veac_ir::{
    channel_samples_for_duration, normalized_video_dimensions, pixel_frames, units_for_duration,
    ProbedStream, Rational, RationalTime, MAX_INPUT_AUDIO_CHANNELS,
};

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) fn video(
    stream: &ProbedStream,
    decode_until: RationalTime,
    extra_frame: bool,
    limits: MediaArtifactLimits,
) -> WorkflowResult<()> {
    let info = stream
        .video
        .as_ref()
        .ok_or_else(|| invalid("selected source stream has no video facts"))?;
    let rate = info
        .frame_rate
        .ok_or_else(|| invalid("selected source video has no canonical frame rate"))?;
    if stream.time_base.is_none() || !rate_within(rate, limits.max_frame_rate) {
        return limit("selected source video timing exceeds the decode budget");
    }
    let normalized = normalized_video_dimensions(info)
        .ok_or_else(|| invalid("selected source video geometry cannot be normalized"))?;
    let width = info.width.max(normalized.0);
    let height = info.height.max(normalized.1);
    let pixels = u64::from(width).saturating_mul(u64::from(height));
    if width > limits.max_dimension
        || height > limits.max_dimension
        || pixels > limits.max_frame_pixels
    {
        return limit("selected source video geometry exceeds the decode budget");
    }
    let mut frames = units(decode_until, rate)?;
    frames = frames.saturating_add(u128::from(extra_frame));
    if frames > limits.max_video_frames
        || pixel_frames(frames, width, height) > limits.max_pixel_frames
    {
        return limit("selected source video work exceeds the decode budget");
    }
    Ok(())
}

pub(super) fn audio(
    stream: &ProbedStream,
    decode_until: RationalTime,
    limits: MediaArtifactLimits,
) -> WorkflowResult<()> {
    let info = stream
        .audio
        .as_ref()
        .ok_or_else(|| invalid("selected source stream has no audio facts"))?;
    if stream.time_base.is_none()
        || info.sample_rate > limits.max_sample_rate
        || info.channels > MAX_INPUT_AUDIO_CHANNELS
    {
        return limit("selected source audio format exceeds the decode budget");
    }
    let samples = channel_samples_for_duration(decode_until, info.sample_rate, info.channels)
        .unwrap_or(u128::MAX);
    if samples > limits.max_audio_channel_samples {
        return limit("selected source audio work exceeds the decode budget");
    }
    Ok(())
}

fn units(duration: RationalTime, rate: Rational) -> WorkflowResult<u128> {
    if duration.value == 0 {
        Ok(0)
    } else {
        units_for_duration(duration, rate)
            .ok_or_else(|| resource("selected source frame calculation overflowed"))
    }
}

fn rate_within(rate: Rational, max: u32) -> bool {
    rate.is_positive()
        && i128::from(rate.numerator) <= i128::from(max) * i128::from(rate.denominator)
}

fn invalid(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::InvalidContract, message)
}

fn resource(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::ResourceLimit, message)
}

fn limit<T>(message: &str) -> WorkflowResult<T> {
    Err(resource(message))
}

#[cfg(test)]
#[path = "work/tests.rs"]
mod tests;
