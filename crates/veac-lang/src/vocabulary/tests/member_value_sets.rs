use crate::authoring::{
    MulticamSyncBasisDecl, ResourceIdentityKind, ResourceLocatorKind, ResourceStreamSelection,
    TemplateFillDecl, TemplateMediaKindDecl, TemplateSlotKindDecl,
};

use super::super::{catalog_core, language_spec, GrammarPosition, VocabularyCategory};
use super::legacy_surface::assert_use_absent;

#[test]
fn core_member_value_sets_are_typed_but_not_published() {
    assert_eq!(catalog_core::RESOURCE_SETS.len(), 3);
    assert_eq!(catalog_core::MEMBER_VALUE_SETS.len(), 4);
    let vocabulary = language_spec().vocabulary;
    for (position, tokens) in [
        (
            GrammarPosition::ResourceLocatorKindPosition,
            ResourceLocatorKind::TOKENS,
        ),
        (
            GrammarPosition::ResourceStreamSelectionPosition,
            ResourceStreamSelection::TOKENS,
        ),
        (
            GrammarPosition::ResourceIdentityKindPosition,
            ResourceIdentityKind::TOKENS,
        ),
        (
            GrammarPosition::MulticamSyncBasisPosition,
            MulticamSyncBasisDecl::TOKENS,
        ),
        (
            GrammarPosition::TemplateSlotKindPosition,
            TemplateSlotKindDecl::TOKENS,
        ),
        (
            GrammarPosition::TemplateMediaKindPosition,
            TemplateMediaKindDecl::TOKENS,
        ),
        (
            GrammarPosition::TemplateFillPosition,
            TemplateFillDecl::TOKENS,
        ),
    ] {
        assert!(!tokens.is_empty());
        for token in tokens {
            assert_use_absent(&vocabulary, token, VocabularyCategory::EnumValue, position);
        }
    }
}
