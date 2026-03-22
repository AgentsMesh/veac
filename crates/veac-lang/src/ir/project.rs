/// Project-level IR types: output format, codec, and quality settings.
use super::FitMode;

#[derive(Debug, Clone)]
pub struct IrProject {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub format: OutputFormat,
    pub codec: Codec,
    pub quality: Quality,
    pub fit: FitMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Mp4,
    Mkv,
    Webm,
    Mov,
}

impl OutputFormat {
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Mp4 => "mp4",
            OutputFormat::Mkv => "mkv",
            OutputFormat::Webm => "webm",
            OutputFormat::Mov => "mov",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Codec {
    H264,
    H265,
    Vp9,
    Av1,
}

impl Codec {
    pub fn ffmpeg_encoder(&self) -> &str {
        match self {
            Codec::H264 => "libx264",
            Codec::H265 => "libx265",
            Codec::Vp9 => "libvpx-vp9",
            Codec::Av1 => "libaom-av1",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    Low,
    Medium,
    High,
    Lossless,
}

impl Quality {
    pub fn ffmpeg_preset(&self) -> &str {
        match self {
            Quality::Low => "ultrafast",
            Quality::Medium => "medium",
            Quality::High => "slow",
            Quality::Lossless => "veryslow",
        }
    }

    pub fn ffmpeg_crf(&self) -> u8 {
        match self {
            Quality::Low => 28,
            Quality::Medium => 23,
            Quality::High => 18,
            Quality::Lossless => 0,
        }
    }
}
