use super::super::{
    language_spec, CanonicalRole, GrammarPosition, IdentifierPolicy, LanguageLayer, SyntaxUse,
    VocabularyCategory, VocabularyEntry,
};

#[test]
fn animate_has_distinct_root_and_component_attachment_uses() {
    let entry = language_spec()
        .vocabulary
        .lookup("animate")
        .unwrap()
        .clone();
    assert!(entry.uses.contains(&SyntaxUse {
        category: VocabularyCategory::ContextualControl,
        layer: LanguageLayer::StaticProgram,
        position: GrammarPosition::StaticDeclaration,
        canonical_role: CanonicalRole::DeclarationIntroducer,
    }));
    assert!(entry.uses.contains(&SyntaxUse {
        category: VocabularyCategory::ContextualControl,
        layer: LanguageLayer::ExecutableExpression,
        position: GrammarPosition::ExpressionTemporalAttachment,
        canonical_role: CanonicalRole::DirectiveIntroducer,
    }));
    assert_eq!(entry.uses.len(), 2);
}

#[test]
fn public_surface_statistics_match_the_machine_contract() {
    let vocabulary = language_spec().vocabulary;
    assert_eq!(vocabulary.entries.len(), 91);
    assert_eq!(
        vocabulary
            .entries
            .iter()
            .map(|entry| entry.uses.len())
            .sum::<usize>(),
        97
    );

    for (category, spellings, uses) in [
        (VocabularyCategory::ReservedLiteral, 2, 2),
        (VocabularyCategory::ContextualControl, 58, 61),
        (VocabularyCategory::EnumValue, 4, 4),
        (VocabularyCategory::BuiltinFunction, 14, 14),
        (VocabularyCategory::UnitSuffix, 6, 6),
        (VocabularyCategory::ValueType, 10, 10),
    ] {
        assert_eq!(vocabulary.count(category), spellings, "{category:?}");
        assert_eq!(
            exact_uses(&vocabulary.entries, category),
            uses,
            "{category:?}"
        );
    }

    for (policy, expected) in [
        (IdentifierPolicy::Allowed, 88),
        (IdentifierPolicy::Reserved, 2),
        (IdentifierPolicy::NotApplicable, 1),
    ] {
        assert_eq!(
            vocabulary
                .entries
                .iter()
                .filter(|entry| entry.identifier_policy == policy)
                .count(),
            expected
        );
    }

    for (layer, expected) in [
        (LanguageLayer::StaticProgram, 59),
        (LanguageLayer::ExecutableExpression, 38),
    ] {
        assert_eq!(
            vocabulary
                .entries
                .iter()
                .flat_map(|entry| &entry.uses)
                .filter(|usage| usage.layer == layer)
                .count(),
            expected
        );
    }
}

fn exact_uses(entries: &[VocabularyEntry], category: VocabularyCategory) -> usize {
    entries
        .iter()
        .flat_map(|entry| &entry.uses)
        .filter(|usage| usage.category == category)
        .count()
}
