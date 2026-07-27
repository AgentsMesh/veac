use veac_ir::{channel_samples_for_duration, RationalTime};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, MediaArtifactLimits};

const WAV_HEADER_ALLOWANCE: u128 = 64 * 1024;

pub(super) fn validate(
    duration: RationalTime,
    sample_rate_value: u32,
    channels: u8,
    uncompressed: bool,
    limits: MediaArtifactLimits,
) -> ArtifactResult<()> {
    sample_rate(sample_rate_value, limits)?;
    let samples =
        channel_samples_for_duration(duration, sample_rate_value, channels).unwrap_or(u128::MAX);
    if samples > limits.max_audio_channel_samples {
        return exceeded("artifact audio work exceeds the channel-sample budget");
    }
    if uncompressed
        && samples
            .saturating_mul(2)
            .saturating_add(WAV_HEADER_ALLOWANCE)
            > u128::from(limits.max_payload_bytes)
    {
        return exceeded("proxy audio estimate exceeds the payload byte budget");
    }
    Ok(())
}

pub(super) fn sample_rate(value: u32, limits: MediaArtifactLimits) -> ArtifactResult<()> {
    if value > limits.max_sample_rate {
        exceeded("artifact sample rate exceeds the resource budget")
    } else {
        Ok(())
    }
}

fn exceeded<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        message,
    ))
}
