use crate::authoring::{
    AdaptivePackageFormat, AlphaMode, AnimatedImageFormat, ArtifactTargetKind,
    AudioChannelLayoutDecl, AudioCodec, AudioFileFormat, AudioMixSourceKind, AudioSelection,
    AudioStemFormat, CaptionOutput, CaptionSidecarFormat, ColorMatrix, ColorPrimaries, ColorRange,
    ColorTransfer, GifDither, GifPlaybackMode, HardwareBackend, HardwareSelection, HlsAudioCodec,
    HlsH264ProfileDecl, HlsVideoCodec, ImageFormat, MuxLayout, OutputChannelLayout, OutputFormat,
    OutputSentinel, PassMode, PixelFormat, VideoCodec, VideoProfile, VideoRateControlKind,
    VideoScope,
};

use super::catalog::TermSet;
use super::GrammarPosition;

pub(super) const SETS: [TermSet; 45] = [
    set(GrammarPosition::OutputFormatPosition, OutputFormat::TOKENS),
    set(GrammarPosition::VideoCodecPosition, VideoCodec::TOKENS),
    set(GrammarPosition::PixelFormatPosition, PixelFormat::TOKENS),
    set(GrammarPosition::AlphaModePosition, AlphaMode::TOKENS),
    set(GrammarPosition::AudioCodecPosition, AudioCodec::TOKENS),
    set(
        GrammarPosition::CaptionOutputPosition,
        CaptionOutput::TOKENS,
    ),
    set(
        GrammarPosition::ArtifactTargetKindPosition,
        ArtifactTargetKind::TOKENS,
    ),
    set(GrammarPosition::MuxLayoutPosition, MuxLayout::TOKENS),
    set(
        GrammarPosition::MuxAudioChannelLayoutPosition,
        OutputChannelLayout::TOKENS,
    ),
    set(
        GrammarPosition::VideoRateControlKindPosition,
        VideoRateControlKind::TOKENS,
    ),
    set(GrammarPosition::PassModePosition, PassMode::TOKENS),
    set(
        GrammarPosition::HardwareBackendPosition,
        HardwareBackend::TOKENS,
    ),
    set(GrammarPosition::ImageFormatPosition, ImageFormat::TOKENS),
    set(
        GrammarPosition::CaptionSidecarFormatPosition,
        CaptionSidecarFormat::TOKENS,
    ),
    set(
        GrammarPosition::AudioStemFormatPosition,
        AudioStemFormat::TOKENS,
    ),
    set(GrammarPosition::VideoScopePosition, VideoScope::TOKENS),
    set(GrammarPosition::GifDitherPosition, GifDither::TOKENS),
    set(GrammarPosition::VideoProfilePosition, VideoProfile::TOKENS),
    set(
        GrammarPosition::ColorPrimariesPosition,
        ColorPrimaries::TOKENS,
    ),
    set(
        GrammarPosition::ColorTransferPosition,
        ColorTransfer::TOKENS,
    ),
    set(GrammarPosition::ColorMatrixPosition, ColorMatrix::TOKENS),
    set(GrammarPosition::ColorRangePosition, ColorRange::TOKENS),
    set(
        GrammarPosition::VideoAudioSelectionPosition,
        AudioSelection::TOKENS,
    ),
    set(
        GrammarPosition::HlsAudioSelectionPosition,
        &OutputSentinel::NONE_TOKENS,
    ),
    set(
        GrammarPosition::HlsAudioCodecPosition,
        HlsAudioCodec::TOKENS,
    ),
    set(
        GrammarPosition::AudioFileChannelLayoutPosition,
        AudioChannelLayoutDecl::TOKENS,
    ),
    set(
        GrammarPosition::HlsAudioChannelLayoutPosition,
        AudioChannelLayoutDecl::TOKENS,
    ),
    set(
        GrammarPosition::AudioFileFormatPosition,
        AudioFileFormat::TOKENS,
    ),
    set(
        GrammarPosition::AnimatedImageFormatPosition,
        AnimatedImageFormat::TOKENS,
    ),
    set(
        GrammarPosition::GifPlaybackValuePosition,
        GifPlaybackMode::TOKENS,
    ),
    set(
        GrammarPosition::AdaptivePackageFormatPosition,
        AdaptivePackageFormat::TOKENS,
    ),
    set(
        GrammarPosition::HlsVideoCodecPosition,
        HlsVideoCodec::TOKENS,
    ),
    set(
        GrammarPosition::HardwareSelectionPosition,
        HardwareSelection::TOKENS,
    ),
    automatic(GrammarPosition::VideoGopPosition),
    automatic(GrammarPosition::VideoBFramesPosition),
    automatic(GrammarPosition::VideoProfileSelectionPosition),
    set(
        GrammarPosition::VideoProfileSelectionPosition,
        VideoProfile::TOKENS,
    ),
    automatic(GrammarPosition::VideoLevelPosition),
    automatic(GrammarPosition::HlsProfilePosition),
    set(
        GrammarPosition::HlsProfilePosition,
        HlsH264ProfileDecl::TOKENS,
    ),
    automatic(GrammarPosition::HlsLevelPosition),
    automatic(GrammarPosition::HlsBFramesPosition),
    set(
        GrammarPosition::VideoColorSpaceSelectionPosition,
        &OutputSentinel::SOURCE_TOKENS,
    ),
    set(
        GrammarPosition::HlsColorSpaceSelectionPosition,
        &OutputSentinel::SOURCE_TOKENS,
    ),
    set(
        GrammarPosition::AudioMixSourceKindPosition,
        AudioMixSourceKind::TOKENS,
    ),
];

const fn set(position: GrammarPosition, tokens: &'static [&'static str]) -> TermSet {
    TermSet::new(position, tokens)
}

const fn automatic(position: GrammarPosition) -> TermSet {
    set(position, &OutputSentinel::AUTOMATIC_TOKENS)
}

#[cfg(test)]
#[path = "tests/catalog_output.rs"]
mod tests;
