use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AudioOutput, BusId, RationalTime, TrackId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageFormat {
    Png,
    Jpeg,
    Tiff,
    Exr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageSequenceOutput {
    pub format: ImageFormat,
    pub start_number: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaptionSidecarFormat {
    Srt,
    WebVtt,
    Ass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionSidecarOutput {
    pub format: CaptionSidecarFormat,
    pub track_ids: Vec<TrackId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioStemFormat {
    Wav,
    Flac,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AudioMixSource {
    Master,
    Track { track_id: TrackId },
    Bus { bus_id: BusId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioStemOutput {
    pub format: AudioStemFormat,
    pub audio: AudioOutput,
    pub source: AudioMixSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoScope {
    Waveform,
    Vectorscope,
    Histogram,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScopeOutput {
    pub scope: VideoScope,
    pub at: RationalTime,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}
