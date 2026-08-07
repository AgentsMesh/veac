use crate::authoring::{
    AudioStateDecl, EditingStateDecl, InterpolationKind, IsolationStateDecl, PlaybackStateDecl,
    SyntaxToken, TextFontKind,
};
use veac_ir::{
    FontStyle, FontWeight, HorizontalTextAlignment, PlacementMode, SourceOutOfRangePolicy,
    TextGranularity, TextOrientation, TextOverflow, TextPathAlignment, TextWrap, TextWritingMode,
    VerticalTextAlignment,
};

use super::catalog::TermSet;
use super::GrammarPosition as P;

pub(super) const SETS: [TermSet; 21] = [
    set(P::TextFontKindPosition, TextFontKind::TOKENS),
    set(P::TextFontWeightPosition, FontWeight::TOKENS),
    set(P::TextFontStylePosition, FontStyle::TOKENS),
    set(P::TextWrapPosition, TextWrap::TOKENS),
    set(P::TextOverflowPosition, TextOverflow::TOKENS),
    set(
        P::TextHorizontalAlignmentPosition,
        HorizontalTextAlignment::TOKENS,
    ),
    set(
        P::TextVerticalAlignmentPosition,
        VerticalTextAlignment::TOKENS,
    ),
    set(P::TextWritingModePosition, TextWritingMode::TOKENS),
    set(P::TextOrientationPosition, TextOrientation::TOKENS),
    set(P::TextPathAlignmentPosition, TextPathAlignment::TOKENS),
    set(P::TextGranularityPosition, TextGranularity::TOKENS),
    set(P::LayerPlacementModePosition, PlacementMode::TOKENS),
    set(P::TrackPlaybackStatePosition, PlaybackStateDecl::TOKENS),
    set(P::ItemPlaybackStatePosition, PlaybackStateDecl::TOKENS),
    set(P::TrackAudioStatePosition, AudioStateDecl::TOKENS),
    set(P::TrackIsolationStatePosition, IsolationStateDecl::TOKENS),
    set(P::TrackEditingStatePosition, EditingStateDecl::TOKENS),
    set(P::MappingOutOfRangePosition, SourceOutOfRangePolicy::TOKENS),
    set(
        P::MappingInterpolationKindPosition,
        InterpolationKind::LEAF_TOKENS,
    ),
    set(
        P::ParameterInterpolationKindPosition,
        InterpolationKind::TOKENS,
    ),
    set(
        P::EffectParameterInterpolationKindPosition,
        InterpolationKind::TOKENS,
    ),
];

const fn set(position: P, tokens: &'static [&'static str]) -> TermSet {
    TermSet::new(position, tokens)
}

#[cfg(test)]
#[path = "tests/catalog_text_timeline.rs"]
mod tests;
