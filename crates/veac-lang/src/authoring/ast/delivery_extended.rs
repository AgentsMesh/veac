mod hls;

pub use hls::*;

use super::{AudioMixSourceDecl, GifDither, ImageFormat, NumberLiteral};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioFileRecipe {
    pub source: AudioMixSourceDecl,
    pub encoding: Mp3EncodingDecl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mp3EncodingDecl {
    pub bitrate: NumberLiteral,
    pub sample_rate: NumberLiteral,
    pub channel_layout: AudioChannelLayoutDecl,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioChannelLayoutDecl {
    Mono,
    Stereo,
}

impl AudioChannelLayoutDecl {
    pub fn count(self) -> u8 {
        match self {
            Self::Mono => 1,
            Self::Stereo => 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimatedImageRecipe {
    pub playback: GifPlaybackDecl,
    pub dither: GifDither,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GifPlaybackDecl {
    Once,
    Forever,
    Times(u16),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StillImageRecipe {
    pub at: NumberLiteral,
    pub format: ImageFormat,
}
