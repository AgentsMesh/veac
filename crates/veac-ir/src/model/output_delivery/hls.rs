use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AudioChannelLayout, AudioMixSource, ColorSpace, HlsRenditionId, RationalTime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "settings",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum AdaptivePackage {
    Hls(HlsPackage),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HlsPackage {
    pub segment_duration: RationalTime,
    pub audio: Option<HlsAudio>,
    pub renditions: Vec<HlsRendition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HlsAudio {
    pub source: AudioMixSource,
    pub encoding: HlsAudioEncoding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "settings",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum HlsAudioEncoding {
    Aac(AacEncoding),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AacEncoding {
    pub bitrate_bps: u32,
    pub sample_rate_hz: u32,
    pub channel_layout: AudioChannelLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HlsRendition {
    pub id: HlsRenditionId,
    pub raster: HlsRenditionRaster,
    pub encoding: HlsVideoEncoding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HlsRenditionRaster {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "settings",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum HlsVideoEncoding {
    H264(HlsH264Encoding),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HlsH264Encoding {
    pub rate_control: HlsCappedBitrate,
    pub profile: Option<HlsH264Profile>,
    pub level: Option<String>,
    pub color_space: Option<ColorSpace>,
    pub b_frames: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HlsCappedBitrate {
    pub target_bps: u64,
    pub max_bps: u64,
    pub buffer_size_bits: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HlsH264Profile {
    Baseline,
    Main,
    High,
}
