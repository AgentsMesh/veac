use veac_ir::{duration_within_seconds, pixel_frames, units_for_duration, Rational, RationalTime};

use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, MediaArtifactLimits, MediaArtifactSpec,
    SourceClockSpec, MAX_ARTIFACT_METADATA_BYTES,
};

mod audio;
mod metadata;

pub(super) fn validate(
    spec: &MediaArtifactSpec,
    limits: MediaArtifactLimits,
) -> ArtifactResult<()> {
    match spec {
        MediaArtifactSpec::ProxyVideo(value) => {
            geometry(value.width, value.height, limits)?;
            clock(value.source_clock, limits)?;
            video(
                value.source_clock.duration(),
                value.frame_rate,
                value.width,
                value.height,
                limits,
            )
        }
        MediaArtifactSpec::ProxyAudio(value) => {
            clock(value.source_clock, limits)?;
            audio::validate(
                value.source_clock.duration(),
                value.sample_rate,
                value.channels,
                true,
                limits,
            )
        }
        MediaArtifactSpec::Waveform(value) => {
            geometry(value.width, value.height, limits)?;
            clock(value.source_clock, limits)?;
            audio::sample_rate(value.sample_rate, limits)?;
            audio::validate(
                value.source_clock.duration(),
                value.sample_rate,
                1,
                false,
                limits,
            )?;
            metadata::bounded_json(&value.color, MAX_ARTIFACT_METADATA_BYTES)
        }
        MediaArtifactSpec::Thumbnail(value) => {
            geometry(value.width, value.height, limits)?;
            time_within(value.at, limits)
        }
        MediaArtifactSpec::OpticalFlow(value) => {
            geometry(value.width, value.height, limits)?;
            clock(value.source_clock, limits)?;
            video(
                value.source_clock.duration(),
                value.frame_rate,
                value.width,
                value.height,
                limits,
            )
        }
        MediaArtifactSpec::Analysis(value) => {
            crate::json::validate_shape(&value.configuration)?;
            metadata::bounded_json(&value.analysis_type, MAX_ARTIFACT_METADATA_BYTES)?;
            metadata::bounded_json(&value.configuration, MAX_ARTIFACT_METADATA_BYTES)
        }
        MediaArtifactSpec::SourceSegment(value) => {
            geometry(value.width, value.height, limits)?;
            range(value.start, value.duration, limits)?;
            video(
                value.duration,
                value.frame_rate,
                value.width,
                value.height,
                limits,
            )?;
            if let Some(audio) = value.audio {
                audio::validate(
                    value.duration,
                    audio.sample_rate,
                    audio.channels,
                    false,
                    limits,
                )?;
            }
            Ok(())
        }
    }
}

fn geometry(width: u32, height: u32, limits: MediaArtifactLimits) -> ArtifactResult<()> {
    let pixels = u64::from(width).saturating_mul(u64::from(height));
    if width > limits.max_dimension
        || height > limits.max_dimension
        || pixels > limits.max_frame_pixels
    {
        return exceeded("artifact frame geometry exceeds the resource budget");
    }
    Ok(())
}

fn clock(value: SourceClockSpec, limits: MediaArtifactLimits) -> ArtifactResult<()> {
    match value {
        SourceClockSpec::Identity { duration } => time_within(duration, limits),
        SourceClockSpec::Bounded { logical_range } => {
            range(logical_range.start, logical_range.duration, limits)
        }
    }
}

fn range(
    start: RationalTime,
    duration: RationalTime,
    limits: MediaArtifactLimits,
) -> ArtifactResult<()> {
    time_within(start, limits)?;
    time_within(duration, limits)?;
    let end = start
        .checked_add(duration)
        .map_err(|_| limit_error("artifact time range overflow"))?;
    time_within(end, limits)
}

fn time_within(value: RationalTime, limits: MediaArtifactLimits) -> ArtifactResult<()> {
    if !value.is_valid()
        || value.value < 0
        || !duration_within_seconds(value, limits.max_duration_seconds)
    {
        return exceeded("artifact time exceeds the duration budget");
    }
    Ok(())
}

fn video(
    duration: RationalTime,
    rate: Rational,
    width: u32,
    height: u32,
    limits: MediaArtifactLimits,
) -> ArtifactResult<()> {
    if u128::try_from(rate.numerator).unwrap_or(u128::MAX)
        > u128::from(limits.max_frame_rate) * u128::from(rate.denominator)
    {
        return exceeded("artifact frame rate exceeds the resource budget");
    }
    let frames = units_for_duration(duration, rate).unwrap_or(u128::MAX);
    if frames > limits.max_video_frames
        || pixel_frames(frames, width, height) > limits.max_pixel_frames
    {
        return exceeded("artifact video work exceeds the resource budget");
    }
    Ok(())
}

fn limit_error(message: &str) -> ArtifactError {
    ArtifactError::new(ArtifactErrorKind::ResourceLimit, message)
}

fn exceeded<T>(message: &str) -> ArtifactResult<T> {
    Err(limit_error(message))
}

#[cfg(test)]
#[path = "budget/tests.rs"]
mod tests;
