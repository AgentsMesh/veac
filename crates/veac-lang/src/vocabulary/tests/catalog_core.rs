use super::{set, KIND_SETS, MEMBER_VALUE_SETS, RELATION_ANNOTATION_SETS, RESOURCE_SETS};
use crate::vocabulary::{language_spec, VocabularyCategory};

#[test]
fn runtime_set_constructor_preserves_core_terms() {
    let expected = &KIND_SETS[0];
    let actual = set(
        std::hint::black_box(expected.context),
        std::hint::black_box(expected.tokens),
    );
    assert_eq!(actual.context, expected.context);
    assert_eq!(actual.tokens, expected.tokens);
}

#[test]
fn legacy_core_catalog_cannot_leak_into_the_language_spec() {
    let vocabulary = language_spec().vocabulary;
    for set in KIND_SETS
        .iter()
        .chain(&RESOURCE_SETS)
        .chain(&MEMBER_VALUE_SETS)
        .chain(&RELATION_ANNOTATION_SETS)
    {
        assert!(!set.context.is_public());
        for spelling in set.tokens {
            assert!(vocabulary.lookup(spelling).is_none_or(|entry| entry
                .uses
                .iter()
                .all(|usage| usage.category != VocabularyCategory::EnumValue
                    || usage.position != set.context)));
        }
    }
}
