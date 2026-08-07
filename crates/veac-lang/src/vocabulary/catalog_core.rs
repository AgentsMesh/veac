use crate::authoring::{
    AnnotationKind, AnnotationTimingKind, ApplyStageKind, ArtifactKind, AudioProcessorKind,
    CardinalDirection, CircleDirection, FadeColor, FillerSuggestion, GeneratorKind, LayerKind,
    ModifierKind, MulticamSyncBasisDecl, RelationKindTag, ResourceIdentityKind, ResourceKind,
    ResourceLocatorKind, ResourceStreamSelection, ReviewAction, SourceKind, SyntaxToken,
    TemplateFillDecl, TemplateMediaKindDecl, TemplateSlotKindDecl, TrackMatteMode,
    TransitionAlignment, TransitionStyleKind, ZoomDirection,
};

use super::catalog::TermSet;
use super::GrammarPosition;

pub(super) const KIND_SETS: [TermSet; 10] = [
    set(GrammarPosition::ResourceKindPosition, ResourceKind::TOKENS),
    set(GrammarPosition::LayerKindPosition, LayerKind::TOKENS),
    set(GrammarPosition::SourceKindPosition, SourceKind::TOKENS),
    set(GrammarPosition::ModifierKindPosition, ModifierKind::TOKENS),
    set(
        GrammarPosition::RelationKindPosition,
        RelationKindTag::TOKENS,
    ),
    set(GrammarPosition::ArtifactKindPosition, ArtifactKind::TOKENS),
    set(
        GrammarPosition::GeneratorKindPosition,
        GeneratorKind::TOKENS,
    ),
    set(
        GrammarPosition::AudioProcessorKindPosition,
        AudioProcessorKind::TOKENS,
    ),
    set(
        GrammarPosition::AnnotationKindPosition,
        AnnotationKind::TOKENS,
    ),
    set(
        GrammarPosition::ApplyStageKindPosition,
        ApplyStageKind::TOKENS,
    ),
];

pub(super) const RESOURCE_SETS: [TermSet; 3] = [
    set(
        GrammarPosition::ResourceLocatorKindPosition,
        ResourceLocatorKind::TOKENS,
    ),
    set(
        GrammarPosition::ResourceStreamSelectionPosition,
        ResourceStreamSelection::TOKENS,
    ),
    set(
        GrammarPosition::ResourceIdentityKindPosition,
        ResourceIdentityKind::TOKENS,
    ),
];

pub(super) const MEMBER_VALUE_SETS: [TermSet; 4] = [
    set(
        GrammarPosition::MulticamSyncBasisPosition,
        MulticamSyncBasisDecl::TOKENS,
    ),
    set(
        GrammarPosition::TemplateSlotKindPosition,
        TemplateSlotKindDecl::TOKENS,
    ),
    set(
        GrammarPosition::TemplateMediaKindPosition,
        TemplateMediaKindDecl::TOKENS,
    ),
    set(
        GrammarPosition::TemplateFillPosition,
        TemplateFillDecl::TOKENS,
    ),
];

pub(super) const RELATION_ANNOTATION_SETS: [TermSet; 11] = [
    set(
        GrammarPosition::TransitionStyleKindPosition,
        TransitionStyleKind::TOKENS,
    ),
    set(
        GrammarPosition::TransitionAlignmentPosition,
        TransitionAlignment::TOKENS,
    ),
    set(GrammarPosition::FadeColorPosition, FadeColor::TOKENS),
    set(
        GrammarPosition::WipeDirectionPosition,
        CardinalDirection::TOKENS,
    ),
    set(
        GrammarPosition::SlideDirectionPosition,
        CardinalDirection::TOKENS,
    ),
    set(
        GrammarPosition::ZoomDirectionPosition,
        ZoomDirection::TOKENS,
    ),
    set(
        GrammarPosition::CircleDirectionPosition,
        CircleDirection::TOKENS,
    ),
    set(GrammarPosition::MatteModePosition, TrackMatteMode::TOKENS),
    set(
        GrammarPosition::AnnotationTimingKindPosition,
        AnnotationTimingKind::TOKENS,
    ),
    set(
        GrammarPosition::FillerSuggestionPosition,
        FillerSuggestion::TOKENS,
    ),
    set(GrammarPosition::ReviewActionPosition, ReviewAction::TOKENS),
];

const fn set(position: GrammarPosition, tokens: &'static [&'static str]) -> TermSet {
    TermSet::new(position, tokens)
}

#[cfg(test)]
#[path = "tests/catalog_core.rs"]
mod tests;
