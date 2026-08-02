use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColorSpace {
    pub primaries: ColorPrimaries,
    pub transfer: ColorTransfer,
    pub matrix: ColorMatrix,
    pub range: ColorRange,
}

impl Default for ColorSpace {
    fn default() -> Self {
        Self {
            primaries: ColorPrimaries::Bt709,
            transfer: ColorTransfer::Bt709,
            matrix: ColorMatrix::Bt709,
            range: ColorRange::Limited,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioOutput {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub channels: u8,
}

impl Default for AudioOutput {
    fn default() -> Self {
        Self {
            codec: AudioCodec::Aac,
            sample_rate: 48_000,
            channels: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VideoRateControl {
    Crf {
        value: u8,
    },
    Bitrate {
        target_bps: u64,
        max_bps: Option<u64>,
        buffer_size_bits: Option<u64>,
    },
    Lossless,
}
