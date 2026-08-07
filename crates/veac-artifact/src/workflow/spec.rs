use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{Rational, RationalTime, StreamSelection, TimeRange};

use crate::{ArtifactKind, ContentDigest, ProducerFingerprint};

mod validate;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MediaArtifactRequest {
    pub source_identity: ContentDigest,
    pub producer: ProducerFingerprint,
    pub spec: MediaArtifactSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum MediaArtifactSpec {
    ProxyVideo(ProxyVideoSpec),
    ProxyAudio(ProxyAudioSpec),
    Waveform(WaveformSpec),
    Thumbnail(ThumbnailSpec),
    OpticalFlow(OpticalFlowSpec),
    SourceSegment(SourceSegmentSpec),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProxyVideoSpec {
    pub source_stream: StreamSelection,
    pub source_clock: SourceClockSpec,
    pub width: u32,
    pub height: u32,
    pub frame_rate: Rational,
    pub crf: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProxyAudioSpec {
    pub source_stream: StreamSelection,
    pub source_clock: SourceClockSpec,
    pub sample_rate: u32,
    pub channels: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceClockSpec {
    Identity { duration: RationalTime },
    Bounded { logical_range: TimeRange },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WaveformSpec {
    pub source_stream: StreamSelection,
    pub source_clock: SourceClockSpec,
    pub sample_rate: u32,
    pub width: u32,
    pub height: u32,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ThumbnailSpec {
    pub source_stream: StreamSelection,
    pub at: RationalTime,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OpticalFlowSpec {
    pub source_stream: StreamSelection,
    pub source_clock: SourceClockSpec,
    pub width: u32,
    pub height: u32,
    pub frame_rate: Rational,
    pub method: OpticalFlowMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpticalFlowMethod {
    BlockMatching,
    MotionCompensated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceSegmentSpec {
    pub video_stream: StreamSelection,
    pub start: RationalTime,
    pub duration: RationalTime,
    pub width: u32,
    pub height: u32,
    pub frame_rate: Rational,
    pub audio: Option<SourceSegmentAudioSpec>,
    pub crf: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceSegmentAudioSpec {
    pub source_stream: StreamSelection,
    pub sample_rate: u32,
    pub channels: u8,
}

impl MediaArtifactSpec {
    pub fn kind(&self) -> ArtifactKind {
        match self {
            Self::ProxyVideo(_) => ArtifactKind::ProxyVideo,
            Self::ProxyAudio(_) => ArtifactKind::ProxyAudio,
            Self::Waveform(_) => ArtifactKind::Waveform,
            Self::Thumbnail(_) => ArtifactKind::Thumbnail,
            Self::OpticalFlow(_) => ArtifactKind::OpticalFlow,
            Self::SourceSegment(_) => ArtifactKind::SourceSegment,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::ProxyVideo(_) | Self::OpticalFlow(_) | Self::SourceSegment(_) => "mp4",
            Self::ProxyAudio(_) => "wav",
            Self::Waveform(_) | Self::Thumbnail(_) => "png",
        }
    }
}

impl SourceClockSpec {
    pub fn duration(self) -> RationalTime {
        match self {
            Self::Identity { duration } => duration,
            Self::Bounded { logical_range } => logical_range.duration,
        }
    }
}
