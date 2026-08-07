use super::super::{
    CanonicalRole, ControlUse, GrammarPosition, LanguageLayer, SyntaxVocabulary, VocabularyCategory,
};

pub(super) fn assert_use_absent(
    vocabulary: &SyntaxVocabulary,
    spelling: &str,
    category: VocabularyCategory,
    position: GrammarPosition,
) {
    assert!(!position.is_public());
    assert!(vocabulary.lookup(spelling).is_none_or(|entry| {
        entry
            .uses
            .iter()
            .all(|usage| usage.category != category || usage.position != position)
    }));
}

pub(super) fn assert_control_absent(vocabulary: &SyntaxVocabulary, control: ControlUse) {
    assert_eq!(control.layer(), LanguageLayer::CoreAuthoring);
    assert_use_absent(
        vocabulary,
        control.as_str(),
        VocabularyCategory::ContextualControl,
        control.position(),
    );
}

pub(super) fn assert_control_contract(
    control: ControlUse,
    position: GrammarPosition,
    role: CanonicalRole,
) {
    assert_eq!(control.position(), position);
    assert_eq!(control.role(), role);
    assert_eq!(control.layer(), LanguageLayer::CoreAuthoring);
}
