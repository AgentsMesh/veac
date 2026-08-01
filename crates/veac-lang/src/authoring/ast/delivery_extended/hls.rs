use super::super::{
    AudioChannelLayoutDecl, AudioMixSourceDecl, ColorSpace, Identifier, NumberLiteral,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdaptivePackageRecipe {
    pub segment_duration: NumberLiteral,
    pub audio: Option<HlsAudioDecl>,
    pub renditions: Vec<HlsRenditionDecl>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HlsAudioDecl {
    pub source: AudioMixSourceDecl,
    pub bitrate: NumberLiteral,
    pub sample_rate: NumberLiteral,
    pub channel_layout: AudioChannelLayoutDecl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HlsRenditionDecl {
    pub id: Identifier,
    pub width: NumberLiteral,
    pub height: NumberLiteral,
    pub encoding: HlsH264EncodingDecl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HlsH264EncodingDecl {
    pub rate_control: HlsCappedBitrateDecl,
    pub profile: Option<HlsH264ProfileDecl>,
    pub level: Option<String>,
    pub color_space: Option<ColorSpace>,
    pub b_frames: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HlsCappedBitrateDecl {
    pub target: NumberLiteral,
    pub max: NumberLiteral,
    pub buffer: NumberLiteral,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HlsH264ProfileDecl {
    Baseline,
    Main,
    High,
}
