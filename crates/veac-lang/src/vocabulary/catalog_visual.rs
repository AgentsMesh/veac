use crate::authoring::SyntaxToken;
use veac_ir::{Anchor, BlendMode, FitMode, HueRange, LutInterpolation, ToneCurveInterpolation};

use super::catalog::TermSet;
use super::GrammarPosition;

pub(super) const SETS: [TermSet; 7] = [
    set(GrammarPosition::ModifierAnchorPosition, Anchor::TOKENS),
    set(GrammarPosition::ModifierFitModePosition, FitMode::TOKENS),
    set(
        GrammarPosition::CompositeBlendModePosition,
        BlendMode::TOKENS,
    ),
    set(
        GrammarPosition::ApplyMixBlendModePosition,
        BlendMode::TOKENS,
    ),
    set(GrammarPosition::HueRangePosition, HueRange::TOKENS),
    set(
        GrammarPosition::ToneCurveInterpolationPosition,
        ToneCurveInterpolation::TOKENS,
    ),
    set(
        GrammarPosition::LutInterpolationPosition,
        LutInterpolation::TOKENS,
    ),
];

const fn set(position: GrammarPosition, tokens: &'static [&'static str]) -> TermSet {
    TermSet::new(position, tokens)
}

#[cfg(test)]
#[path = "tests/catalog_visual.rs"]
mod tests;
