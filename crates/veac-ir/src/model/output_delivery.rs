use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod hls;

pub use hls::*;

use crate::{AudioMixSource, ImageFormat, RationalTime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioFile {
    pub source: AudioMixSource,
    pub encoding: AudioFileEncoding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "settings",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum AudioFileEncoding {
    Mp3(Mp3Encoding),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mp3Encoding {
    pub bitrate_bps: u32,
    pub sample_rate_hz: u32,
    pub channel_layout: AudioChannelLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioChannelLayout {
    Mono,
    Stereo,
}

impl AudioChannelLayout {
    pub fn count(self) -> u8 {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "settings",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum AnimatedImage {
    Gif(GifAnimation),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GifAnimation {
    pub playback: GifPlayback,
    pub dither: GifDither,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum GifPlayback {
    Once,
    Forever,
    Times { count: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GifDither {
    Bayer,
    FloydSteinberg,
    Sierra2,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StillImage {
    pub frame: FrameSelection,
    pub encoding: ImageFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum FrameSelection {
    Containing { at: RationalTime },
}
