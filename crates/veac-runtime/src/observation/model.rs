use std::path::PathBuf;

use veac_ir::{MediaIdentity, RationalTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationSource {
    pub path: PathBuf,
    pub identity: MediaIdentity,
    pub video_stream: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FramePixelFormat {
    Rgb8,
    Rgba8,
    Alpha16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameRequest {
    pub source: ObservationSource,
    pub time: RationalTime,
    pub pixel_format: FramePixelFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeRequest {
    pub source: ObservationSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedFrame {
    pub requested_time: RationalTime,
    pub actual_pts: RationalTime,
    pub width: u32,
    pub height: u32,
    pub pixel_format: FramePixelFormat,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeObservation {
    pub complete: bool,
    pub identity: MediaIdentity,
    pub decoded_frames: u64,
    pub last_pts: Option<RationalTime>,
    pub errors: Vec<String>,
}

impl FramePixelFormat {
    pub(crate) const fn ffmpeg_name(self) -> &'static str {
        match self {
            Self::Rgb8 => "rgb24",
            Self::Rgba8 => "rgba",
            Self::Alpha16 => "gray16le",
        }
    }

    pub(crate) const fn bytes_per_pixel(self) -> u8 {
        match self {
            Self::Rgb8 => 3,
            Self::Rgba8 => 4,
            Self::Alpha16 => 2,
        }
    }
}
