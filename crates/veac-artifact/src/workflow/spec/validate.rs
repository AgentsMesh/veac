use veac_ir::RationalTime;

use super::{MediaArtifactRequest, MediaArtifactSpec};
use crate::{
    ArtifactDependency, ArtifactDependencyRole, ArtifactDescriptor, ArtifactError,
    ArtifactErrorKind, ArtifactParameters, ArtifactResult, MediaArtifactLimits,
};

mod budget;

impl MediaArtifactRequest {
    pub fn descriptor(&self) -> ArtifactResult<ArtifactDescriptor> {
        self.validate()?;
        let parameters = match &self.spec {
            MediaArtifactSpec::ProxyVideo(value) => ArtifactParameters::ProxyVideo(value.clone()),
            MediaArtifactSpec::ProxyAudio(value) => ArtifactParameters::ProxyAudio(value.clone()),
            MediaArtifactSpec::Waveform(value) => ArtifactParameters::Waveform(value.clone()),
            MediaArtifactSpec::Thumbnail(value) => ArtifactParameters::Thumbnail(value.clone()),
            MediaArtifactSpec::OpticalFlow(value) => ArtifactParameters::OpticalFlow(value.clone()),
            MediaArtifactSpec::SourceSegment(value) => {
                ArtifactParameters::SourceSegment(value.clone())
            }
        };
        let descriptor = ArtifactDescriptor::new(
            self.producer.clone(),
            vec![ArtifactDependency::new(
                ArtifactDependencyRole::Input,
                self.source_identity.clone(),
            )],
            parameters,
        );
        descriptor.validate()?;
        Ok(descriptor)
    }

    pub fn validate(&self) -> ArtifactResult<()> {
        self.validate_with_limits(MediaArtifactLimits::default())
    }

    pub fn validate_with_limits(&self, limits: MediaArtifactLimits) -> ArtifactResult<()> {
        if !limits.is_valid() {
            return invalid("media artifact resource limits are invalid");
        }
        self.source_identity.validate()?;
        if self.producer.name.is_empty() || self.producer.version.is_empty() {
            return invalid("media artifact producer is incomplete");
        }
        self.producer.configuration.validate()?;
        self.spec.validate_contract()?;
        budget::validate(&self.spec, limits)
    }
}

impl MediaArtifactSpec {
    pub(crate) fn validate_descriptor_parameters(&self) -> ArtifactResult<()> {
        self.validate_contract()?;
        budget::validate(self, MediaArtifactLimits::default())
    }

    fn validate_contract(&self) -> ArtifactResult<()> {
        match self {
            Self::ProxyVideo(value) => {
                dimensions(value.width, value.height, true)?;
                value.source_clock.validate()?;
                rate(value.frame_rate, "proxy video frame rate")?;
                if value.crf > 51 {
                    return invalid("proxy video CRF must be at most 51");
                }
            }
            Self::ProxyAudio(value) => {
                value.source_clock.validate()?;
                audio(value.sample_rate, value.channels)?;
            }
            Self::Waveform(value) => {
                dimensions(value.width, value.height, false)?;
                value.source_clock.validate()?;
                if value.sample_rate == 0
                    || value.color.is_empty()
                    || !value.color.bytes().all(valid_color_byte)
                {
                    return invalid("waveform settings are invalid");
                }
            }
            Self::Thumbnail(value) => {
                dimensions(value.width, value.height, false)?;
                nonnegative(value.at, "thumbnail time")?;
                exact_backend_time(value.at, "thumbnail time")?;
            }
            Self::OpticalFlow(value) => {
                dimensions(value.width, value.height, true)?;
                if value.width < 32 || value.height < 32 {
                    return invalid("optical-flow dimensions must be at least 32x32");
                }
                value.source_clock.validate()?;
                rate(value.frame_rate, "optical-flow frame rate")?;
            }
            Self::SourceSegment(value) => {
                dimensions(value.width, value.height, true)?;
                nonnegative(value.start, "source segment start")?;
                positive(value.duration, "source segment duration")?;
                exact_backend_time(value.start, "source segment start")?;
                exact_backend_time(value.duration, "source segment duration")?;
                rate(value.frame_rate, "source segment frame rate")?;
                if value.crf > 51 {
                    return invalid("source segment CRF must be at most 51");
                }
                if let Some(settings) = value.audio {
                    audio(settings.sample_rate, settings.channels)?;
                }
            }
        }
        Ok(())
    }
}

impl super::SourceClockSpec {
    pub fn validate(self) -> ArtifactResult<()> {
        match self {
            Self::Identity { duration } => {
                positive(duration, "proxy source duration")?;
                exact_backend_time(duration, "proxy source duration")
            }
            Self::Bounded { logical_range }
                if logical_range.start.is_valid()
                    && logical_range.duration.is_valid()
                    && logical_range.start.value >= 0
                    && logical_range.duration.value > 0
                    && logical_range.start.timescale == logical_range.duration.timescale
                    && logical_range.end().is_ok() =>
            {
                exact_backend_time(logical_range.start, "proxy source start")?;
                exact_backend_time(logical_range.duration, "proxy source duration")
            }
            _ => invalid("proxy source clock must describe an exact zero-origin artifact"),
        }
    }
}

fn dimensions(width: u32, height: u32, encoded_video: bool) -> ArtifactResult<()> {
    if width == 0 || height == 0 || (encoded_video && (width % 2 != 0 || height % 2 != 0)) {
        invalid("artifact dimensions are invalid for the output format")
    } else {
        Ok(())
    }
}

fn audio(sample_rate: u32, channels: u8) -> ArtifactResult<()> {
    if sample_rate == 0 || !(1..=8).contains(&channels) {
        invalid("artifact audio settings are invalid")
    } else {
        Ok(())
    }
}

fn rate(value: veac_ir::Rational, name: &str) -> ArtifactResult<()> {
    if value.is_positive() {
        Ok(())
    } else {
        invalid(&format!("{name} must be positive and canonical"))
    }
}

fn nonnegative(value: RationalTime, name: &str) -> ArtifactResult<()> {
    if !value.is_valid() || value.value < 0 {
        invalid(&format!("{name} must be a nonnegative exact time"))
    } else {
        Ok(())
    }
}

fn positive(value: RationalTime, name: &str) -> ArtifactResult<()> {
    if !value.is_valid() || value.value <= 0 {
        invalid(&format!("{name} must be a positive exact time"))
    } else {
        Ok(())
    }
}

fn exact_backend_time(value: RationalTime, name: &str) -> ArtifactResult<()> {
    let scaled = i128::from(value.value).checked_mul(1_000_000);
    if scaled.is_none_or(|scaled| scaled % i128::from(value.timescale) != 0) {
        return invalid(&format!(
            "{name} must be exactly representable at FFmpeg microsecond precision"
        ));
    }
    Ok(())
}

fn valid_color_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'#' | b'@' | b'_' | b'-')
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}
