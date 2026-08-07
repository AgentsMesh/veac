use crate::authoring::{ApplyScopeKind, MappingKind};
use crate::vocabulary::{GrammarPosition as P, SyntaxVocabulary, VocabularyCategory};

use super::super::{catalog_structure, language_spec};
use super::legacy_surface::assert_use_absent;

#[test]
fn mapping_and_apply_legacy_kinds_are_not_published() {
    assert_eq!(catalog_structure::SETS.len(), 2);
    let vocabulary = language_spec().vocabulary;
    assert_exact(&vocabulary, MappingKind::TOKENS, P::MappingKindPosition);
    assert_exact(
        &vocabulary,
        ApplyScopeKind::TOKENS,
        P::ApplyScopeKindPosition,
    );
}

fn assert_exact(vocabulary: &SyntaxVocabulary, tokens: &[&str], position: P) {
    for token in tokens {
        assert_use_absent(vocabulary, token, VocabularyCategory::EnumValue, position);
    }
}
