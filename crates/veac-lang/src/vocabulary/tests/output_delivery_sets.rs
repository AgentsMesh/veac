use crate::authoring::{
    AdaptivePackageFormat, AnimatedImageFormat, ArtifactTargetKind, AudioChannelLayoutDecl,
    AudioFileFormat, GifPlaybackMode, HlsVideoCodec, MuxLayout, OutputChannelLayout,
    VideoRateControlKind,
};

use super::super::{language_spec, GrammarPosition, VocabularyCategory};
use super::legacy_surface::assert_use_absent;

#[test]
fn delivery_recipe_descriptors_are_not_surface_vocabulary() {
    let vocabulary = language_spec().vocabulary;
    for (token, position) in [
        (
            AudioFileFormat::Mp3.as_str(),
            GrammarPosition::AudioFileFormatPosition,
        ),
        (
            AnimatedImageFormat::Gif.as_str(),
            GrammarPosition::AnimatedImageFormatPosition,
        ),
        (
            AdaptivePackageFormat::Hls.as_str(),
            GrammarPosition::AdaptivePackageFormatPosition,
        ),
        (
            HlsVideoCodec::H264.as_str(),
            GrammarPosition::HlsVideoCodecPosition,
        ),
    ] {
        assert_use_absent(&vocabulary, token, VocabularyCategory::EnumValue, position);
    }
}

#[test]
fn video_model_descriptors_are_not_surface_vocabulary() {
    let vocabulary = language_spec().vocabulary;
    for (tokens, position) in [
        (MuxLayout::TOKENS, GrammarPosition::MuxLayoutPosition),
        (
            ArtifactTargetKind::TOKENS,
            GrammarPosition::ArtifactTargetKindPosition,
        ),
        (
            OutputChannelLayout::TOKENS,
            GrammarPosition::MuxAudioChannelLayoutPosition,
        ),
        (
            VideoRateControlKind::TOKENS,
            GrammarPosition::VideoRateControlKindPosition,
        ),
        (
            AudioChannelLayoutDecl::TOKENS,
            GrammarPosition::AudioFileChannelLayoutPosition,
        ),
        (
            AudioChannelLayoutDecl::TOKENS,
            GrammarPosition::HlsAudioChannelLayoutPosition,
        ),
        (
            GifPlaybackMode::TOKENS,
            GrammarPosition::GifPlaybackValuePosition,
        ),
    ] {
        for token in tokens {
            assert_use_absent(&vocabulary, token, VocabularyCategory::EnumValue, position);
        }
    }
    assert_use_absent(
        &vocabulary,
        "times",
        VocabularyCategory::UnitSuffix,
        GrammarPosition::GifPlaybackValuePosition,
    );
}
