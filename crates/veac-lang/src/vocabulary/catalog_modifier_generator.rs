use crate::authoring::{
    ColorCurveChannel, ColorStageKind, GradientGeometryKind, MaskShapeKind, PaintKind,
    PlacementKind, ShapeGeometryKind, SyntaxToken,
};
use veac_ir::{AudioFadeCurve, PitchPolicy};

use super::catalog::TermSet;
use super::GrammarPosition as P;

pub(super) const SETS: [TermSet; 9] = [
    set(P::PlacementKindPosition, PlacementKind::TOKENS),
    set(P::MaskShapeKindPosition, MaskShapeKind::TOKENS),
    set(P::PitchPolicyPosition, PitchPolicy::TOKENS),
    set(P::AudioFadeCurvePosition, AudioFadeCurve::TOKENS),
    set(P::ColorStageKindPosition, ColorStageKind::TOKENS),
    set(P::ColorCurveChannelPosition, ColorCurveChannel::TOKENS),
    set(
        P::GradientGeometryKindPosition,
        GradientGeometryKind::TOKENS,
    ),
    set(P::ShapeGeometryKindPosition, ShapeGeometryKind::TOKENS),
    set(P::PaintKindPosition, PaintKind::TOKENS),
];

const fn set(position: P, tokens: &'static [&'static str]) -> TermSet {
    TermSet::new(position, tokens)
}

#[cfg(test)]
#[path = "tests/catalog_modifier_generator.rs"]
mod tests;
