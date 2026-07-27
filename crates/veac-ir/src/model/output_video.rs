use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AudioOutput, CaptionOutput, ColorSpace, OutputFormat};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VideoDeliverable {
    pub container: OutputFormat,
    pub video: VideoOutput,
    pub audio: Option<AudioOutput>,
    pub captions: CaptionOutput,
    pub optimize_for_streaming: bool,
    pub pass_mode: PassMode,
    pub hardware: HardwareSelection,
}

impl Default for VideoDeliverable {
    fn default() -> Self {
        Self {
            container: OutputFormat::Mp4,
            video: VideoOutput::default(),
            audio: Some(AudioOutput {
                codec: crate::AudioCodec::Aac,
                sample_rate: 48_000,
                channels: 2,
            }),
            captions: CaptionOutput::BurnIn,
            optimize_for_streaming: false,
            pass_mode: PassMode::Single,
            hardware: HardwareSelection::Auto,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VideoOutput {
    pub codec: VideoCodec,
    pub pixel_format: PixelFormat,
    pub alpha: AlphaMode,
    pub color_space: Option<ColorSpace>,
    pub rate_control: VideoRateControl,
    pub gop_size: Option<u32>,
    pub b_frames: Option<u8>,
    pub profile: Option<VideoProfile>,
    pub level: Option<String>,
}

impl Default for VideoOutput {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            pixel_format: PixelFormat::Yuv420p,
            alpha: AlphaMode::Opaque,
            color_space: None,
            rate_control: VideoRateControl::Crf { value: 23 },
            gop_size: None,
            b_frames: None,
            profile: None,
            level: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum VideoRateControl {
    Crf {
        value: u8,
    },
    Bitrate {
        #[schemars(range(max = 9007199254740991u64))]
        target_bps: u64,
        #[schemars(range(max = 9007199254740991u64))]
        max_bps: Option<u64>,
        #[schemars(range(max = 9007199254740991u64))]
        buffer_bps: Option<u64>,
    },
    Lossless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PixelFormat {
    Yuv420p,
    Yuv420p10le,
    Yuv422p,
    Yuv422p10le,
    Yuv444p10le,
    Yuva444p10le,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AlphaMode {
    Opaque,
    Straight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoProfile {
    H264Baseline,
    H264Main,
    H264High,
    H264High10,
    H265Main,
    H265Main10,
    Vp9Profile0,
    Vp9Profile2,
    Av1Main,
    ProRes4444,
    DnxHrLb,
    DnxHrSq,
    DnxHrHq,
    DnxHrHqx,
    DnxHr444,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
    H265,
    Vp9,
    Av1,
    ProRes,
    DnxHr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PassMode {
    Single,
    TwoPass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum HardwareSelection {
    Auto,
    Software,
    Explicit { backend: HardwareBackend },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HardwareBackend {
    VideoToolbox,
    Nvenc,
    Qsv,
    Vaapi,
}
