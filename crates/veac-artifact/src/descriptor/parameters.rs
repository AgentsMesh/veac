use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AnalysisArtifactParameters, ArtifactKind, ArtifactResult, OpticalFlowSpec, ProxyAudioSpec,
    ProxyVideoSpec, SourceSegmentSpec, ThumbnailSpec, WaveformSpec,
};

mod provider;
mod render;

pub use provider::*;
pub use render::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ArtifactParameters {
    ProxyVideo(ProxyVideoSpec),
    ProxyAudio(ProxyAudioSpec),
    Waveform(WaveformSpec),
    Thumbnail(ThumbnailSpec),
    Speech(ProviderResultParameters),
    Translation(ProviderResultParameters),
    MotionTrack(ProviderResultParameters),
    Matte(ProviderResultParameters),
    OpticalFlow(OpticalFlowSpec),
    Analysis(AnalysisArtifactParameters),
    SourceSegment(SourceSegmentSpec),
    RenderSegment(RenderSegmentParameters),
    CaptionSidecar(RenderOutputParameters),
    AudioStem(ProducedArtifactParameters),
    AudioFile(RenderOutputParameters),
    AnimatedImage(RenderOutputParameters),
    StillImage(RenderOutputParameters),
    AdaptivePackage(RenderOutputParameters),
    VideoMaster(ProducedArtifactParameters),
    ImageSequenceFrame(RenderOutputParameters),
    VideoWaveform(RenderOutputParameters),
    Vectorscope(RenderOutputParameters),
    Histogram(RenderOutputParameters),
    RenderCheckpoint(RenderCheckpointParameters),
}

impl ArtifactParameters {
    pub fn provider_result(
        kind: ArtifactKind,
        parameters: ProviderResultParameters,
    ) -> ArtifactResult<Self> {
        let value = match kind {
            ArtifactKind::Speech => Self::Speech(parameters),
            ArtifactKind::Translation => Self::Translation(parameters),
            ArtifactKind::MotionTrack => Self::MotionTrack(parameters),
            ArtifactKind::Matte => Self::Matte(parameters),
            ArtifactKind::AudioStem => {
                Self::AudioStem(ProducedArtifactParameters::Provider(parameters))
            }
            ArtifactKind::VideoMaster => {
                Self::VideoMaster(ProducedArtifactParameters::Provider(parameters))
            }
            _ => return super::invalid("artifact kind is not a provider result"),
        };
        Ok(value)
    }

    pub fn kind(&self) -> ArtifactKind {
        match self {
            Self::ProxyVideo(_) => ArtifactKind::ProxyVideo,
            Self::ProxyAudio(_) => ArtifactKind::ProxyAudio,
            Self::Waveform(_) => ArtifactKind::Waveform,
            Self::Thumbnail(_) => ArtifactKind::Thumbnail,
            Self::Speech(_) => ArtifactKind::Speech,
            Self::Translation(_) => ArtifactKind::Translation,
            Self::MotionTrack(_) => ArtifactKind::MotionTrack,
            Self::Matte(_) => ArtifactKind::Matte,
            Self::OpticalFlow(_) => ArtifactKind::OpticalFlow,
            Self::Analysis(_) => ArtifactKind::Analysis,
            Self::SourceSegment(_) => ArtifactKind::SourceSegment,
            Self::RenderSegment(_) => ArtifactKind::RenderSegment,
            Self::CaptionSidecar(_) => ArtifactKind::CaptionSidecar,
            Self::AudioStem(_) => ArtifactKind::AudioStem,
            Self::AudioFile(_) => ArtifactKind::AudioFile,
            Self::AnimatedImage(_) => ArtifactKind::AnimatedImage,
            Self::StillImage(_) => ArtifactKind::StillImage,
            Self::AdaptivePackage(_) => ArtifactKind::AdaptivePackage,
            Self::VideoMaster(_) => ArtifactKind::VideoMaster,
            Self::ImageSequenceFrame(_) => ArtifactKind::ImageSequenceFrame,
            Self::VideoWaveform(_) => ArtifactKind::VideoWaveform,
            Self::Vectorscope(_) => ArtifactKind::Vectorscope,
            Self::Histogram(_) => ArtifactKind::Histogram,
            Self::RenderCheckpoint(_) => ArtifactKind::RenderCheckpoint,
        }
    }

    pub fn provider_slot(&self) -> Option<&ProviderArtifactSlot> {
        let value = match self {
            Self::Speech(value)
            | Self::Translation(value)
            | Self::MotionTrack(value)
            | Self::Matte(value) => value,
            Self::AudioStem(ProducedArtifactParameters::Provider(value))
            | Self::VideoMaster(ProducedArtifactParameters::Provider(value)) => value,
            _ => return None,
        };
        Some(&value.slot)
    }

    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        match self {
            Self::Speech(value)
            | Self::Translation(value)
            | Self::MotionTrack(value)
            | Self::Matte(value) => value.validate(),
            Self::Analysis(value) => value.validate(),
            Self::RenderSegment(value) => value.validate(),
            Self::CaptionSidecar(value)
            | Self::AudioFile(value)
            | Self::AnimatedImage(value)
            | Self::StillImage(value)
            | Self::AdaptivePackage(value)
            | Self::ImageSequenceFrame(value)
            | Self::VideoWaveform(value)
            | Self::Vectorscope(value)
            | Self::Histogram(value) => value.validate(),
            Self::AudioStem(value) | Self::VideoMaster(value) => value.validate(),
            Self::RenderCheckpoint(value) => value.validate(),
            Self::ProxyVideo(value) => {
                crate::MediaArtifactSpec::ProxyVideo(value.clone()).validate_descriptor_parameters()
            }
            Self::ProxyAudio(value) => {
                crate::MediaArtifactSpec::ProxyAudio(value.clone()).validate_descriptor_parameters()
            }
            Self::Waveform(value) => {
                crate::MediaArtifactSpec::Waveform(value.clone()).validate_descriptor_parameters()
            }
            Self::Thumbnail(value) => {
                crate::MediaArtifactSpec::Thumbnail(value.clone()).validate_descriptor_parameters()
            }
            Self::OpticalFlow(value) => crate::MediaArtifactSpec::OpticalFlow(value.clone())
                .validate_descriptor_parameters(),
            Self::SourceSegment(value) => crate::MediaArtifactSpec::SourceSegment(value.clone())
                .validate_descriptor_parameters(),
        }
    }
}
