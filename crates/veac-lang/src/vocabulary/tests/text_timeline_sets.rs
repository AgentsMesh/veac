use crate::authoring::{
    AudioStateDecl, EditingStateDecl, InterpolationKind, IsolationStateDecl, PlaybackStateDecl,
    SyntaxToken, TextFontKind,
};
use crate::vocabulary::SyntaxVocabulary;
use veac_ir::{
    FontStyle, FontWeight, HorizontalTextAlignment, PlacementMode, SourceOutOfRangePolicy,
    TextGranularity, TextOrientation, TextOverflow, TextPathAlignment, TextWrap, TextWritingMode,
    VerticalTextAlignment,
};

use super::super::{
    catalog_text_timeline, language_spec, GrammarPosition as P, VocabularyCategory,
};
use super::legacy_surface::assert_use_absent;

#[test]
fn text_timeline_and_mapping_legacy_sets_are_not_published() {
    assert_eq!(catalog_text_timeline::SETS.len(), 21);
    let vocabulary = language_spec().vocabulary;
    for (tokens, position) in [
        (TextFontKind::TOKENS, P::TextFontKindPosition),
        (FontWeight::TOKENS, P::TextFontWeightPosition),
        (FontStyle::TOKENS, P::TextFontStylePosition),
        (TextWrap::TOKENS, P::TextWrapPosition),
        (TextOverflow::TOKENS, P::TextOverflowPosition),
        (
            HorizontalTextAlignment::TOKENS,
            P::TextHorizontalAlignmentPosition,
        ),
        (
            VerticalTextAlignment::TOKENS,
            P::TextVerticalAlignmentPosition,
        ),
        (TextWritingMode::TOKENS, P::TextWritingModePosition),
        (TextOrientation::TOKENS, P::TextOrientationPosition),
        (TextPathAlignment::TOKENS, P::TextPathAlignmentPosition),
        (TextGranularity::TOKENS, P::TextGranularityPosition),
        (PlacementMode::TOKENS, P::LayerPlacementModePosition),
        (PlaybackStateDecl::TOKENS, P::TrackPlaybackStatePosition),
        (PlaybackStateDecl::TOKENS, P::ItemPlaybackStatePosition),
        (AudioStateDecl::TOKENS, P::TrackAudioStatePosition),
        (IsolationStateDecl::TOKENS, P::TrackIsolationStatePosition),
        (EditingStateDecl::TOKENS, P::TrackEditingStatePosition),
        (SourceOutOfRangePolicy::TOKENS, P::MappingOutOfRangePosition),
        (
            InterpolationKind::LEAF_TOKENS,
            P::MappingInterpolationKindPosition,
        ),
        (
            InterpolationKind::TOKENS,
            P::ParameterInterpolationKindPosition,
        ),
        (
            InterpolationKind::TOKENS,
            P::EffectParameterInterpolationKindPosition,
        ),
    ] {
        assert_exact(&vocabulary, tokens, position);
    }
}

#[test]
fn mapping_interpolation_is_the_exact_leaf_subset() {
    let leaf = [
        InterpolationKind::Hold,
        InterpolationKind::Linear,
        InterpolationKind::EaseIn,
        InterpolationKind::EaseOut,
        InterpolationKind::EaseInOut,
    ];
    for (kind, token) in leaf.iter().zip(InterpolationKind::LEAF_TOKENS) {
        assert_eq!(kind.as_str(), *token);
    }
    assert!(!leaf.contains(&InterpolationKind::Spring));
    assert!(!leaf.contains(&InterpolationKind::CubicBezier));
}

fn assert_exact(vocabulary: &SyntaxVocabulary, tokens: &[&str], position: P) {
    for token in tokens {
        assert_use_absent(vocabulary, token, VocabularyCategory::EnumValue, position);
    }
}
