use crate::authoring::SyntaxToken;
use crate::vocabulary::SyntaxVocabulary;
use veac_ir::{Anchor, BlendMode, FitMode, HueRange, LutInterpolation, ToneCurveInterpolation};

use super::super::{catalog_visual, language_spec, GrammarPosition, VocabularyCategory};
use super::legacy_surface::assert_use_absent;

#[test]
fn visual_and_color_legacy_descriptors_are_not_published() {
    assert_eq!(catalog_visual::SETS.len(), 7);
    let vocabulary = language_spec().vocabulary;
    for (tokens, position) in [
        (Anchor::TOKENS, GrammarPosition::ModifierAnchorPosition),
        (FitMode::TOKENS, GrammarPosition::ModifierFitModePosition),
        (
            BlendMode::TOKENS,
            GrammarPosition::CompositeBlendModePosition,
        ),
        (
            BlendMode::TOKENS,
            GrammarPosition::ApplyMixBlendModePosition,
        ),
        (HueRange::TOKENS, GrammarPosition::HueRangePosition),
        (
            ToneCurveInterpolation::TOKENS,
            GrammarPosition::ToneCurveInterpolationPosition,
        ),
        (
            LutInterpolation::TOKENS,
            GrammarPosition::LutInterpolationPosition,
        ),
    ] {
        assert_exact(&vocabulary, tokens, position);
    }
}

fn assert_exact(vocabulary: &SyntaxVocabulary, tokens: &[&str], position: GrammarPosition) {
    for token in tokens {
        assert_use_absent(vocabulary, token, VocabularyCategory::EnumValue, position);
    }
}
