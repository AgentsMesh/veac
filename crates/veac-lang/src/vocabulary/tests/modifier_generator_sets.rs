use crate::authoring::{
    ColorCurveChannel, ColorStageKind, GradientGeometryKind, MaskShapeKind, PaintKind,
    PlacementKind, ShapeGeometryKind, SyntaxToken,
};
use crate::vocabulary::{GrammarPosition as P, SyntaxVocabulary, VocabularyCategory};
use veac_ir::{AudioFadeCurve, PitchPolicy};

use super::super::{catalog_modifier_generator, language_spec};
use super::legacy_surface::assert_use_absent;

#[test]
fn modifier_and_generator_legacy_descriptors_are_not_published() {
    assert_eq!(catalog_modifier_generator::SETS.len(), 9);
    let vocabulary = language_spec().vocabulary;
    for (tokens, position) in [
        (PlacementKind::TOKENS, P::PlacementKindPosition),
        (MaskShapeKind::TOKENS, P::MaskShapeKindPosition),
        (PitchPolicy::TOKENS, P::PitchPolicyPosition),
        (AudioFadeCurve::TOKENS, P::AudioFadeCurvePosition),
        (ColorStageKind::TOKENS, P::ColorStageKindPosition),
        (ColorCurveChannel::TOKENS, P::ColorCurveChannelPosition),
        (
            GradientGeometryKind::TOKENS,
            P::GradientGeometryKindPosition,
        ),
        (ShapeGeometryKind::TOKENS, P::ShapeGeometryKindPosition),
        (PaintKind::TOKENS, P::PaintKindPosition),
    ] {
        assert_exact(&vocabulary, tokens, position);
    }
}

fn assert_exact(vocabulary: &SyntaxVocabulary, tokens: &[&str], position: P) {
    for token in tokens {
        assert_use_absent(vocabulary, token, VocabularyCategory::EnumValue, position);
    }
}
