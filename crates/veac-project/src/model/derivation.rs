use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{InputId, MediaType, ProjectRational};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum MediaDerivation {
    ProxyVideo {
        source: InputId,
        source_stream: ProjectStreamSelection,
        source_clock: ProjectSourceClock,
        width: u32,
        height: u32,
        frame_rate: ProjectRational,
        crf: u8,
    },
    ProxyAudio {
        source: InputId,
        source_stream: ProjectStreamSelection,
        source_clock: ProjectSourceClock,
        sample_rate: u32,
        channels: u8,
    },
    Thumbnail {
        source: InputId,
        source_stream: ProjectStreamSelection,
        at: ProjectRational,
        width: u32,
        height: u32,
    },
    Waveform {
        source: InputId,
        source_stream: ProjectStreamSelection,
        source_clock: ProjectSourceClock,
        sample_rate: u32,
        width: u32,
        height: u32,
        color: String,
    },
    OpticalFlow {
        source: InputId,
        source_stream: ProjectStreamSelection,
        source_clock: ProjectSourceClock,
        width: u32,
        height: u32,
        frame_rate: ProjectRational,
        method: ProjectOpticalFlowMethod,
    },
    SourceSegment {
        source: InputId,
        video_stream: ProjectStreamSelection,
        start: ProjectRational,
        duration: ProjectRational,
        width: u32,
        height: u32,
        frame_rate: ProjectRational,
        audio: Option<ProjectSegmentAudio>,
        crf: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectStreamSelection {
    pub global_index: u32,
    pub type_index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectSourceClock {
    Identity {
        duration: ProjectRational,
    },
    Bounded {
        start: ProjectRational,
        duration: ProjectRational,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProjectOpticalFlowMethod {
    BlockMatching,
    MotionCompensated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectSegmentAudio {
    pub source_stream: ProjectStreamSelection,
    pub sample_rate: u32,
    pub channels: u8,
}

impl MediaDerivation {
    pub fn source(&self) -> &InputId {
        match self {
            Self::ProxyVideo { source, .. }
            | Self::ProxyAudio { source, .. }
            | Self::Thumbnail { source, .. }
            | Self::Waveform { source, .. }
            | Self::OpticalFlow { source, .. }
            | Self::SourceSegment { source, .. } => source,
        }
    }

    pub fn output_media_type(&self) -> MediaType {
        match self {
            Self::ProxyVideo { .. } | Self::OpticalFlow { .. } | Self::SourceSegment { .. } => {
                MediaType::Video
            }
            Self::ProxyAudio { .. } => MediaType::Audio,
            Self::Thumbnail { .. } | Self::Waveform { .. } => MediaType::Image,
        }
    }
}
