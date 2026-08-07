use super::super::{
    language_spec, CanonicalRole, GrammarPosition, IdentifierPolicy, LanguageLayer, SyntaxUse,
    SyntaxVocabulary, VocabularyCategory,
};

#[test]
fn validation_rejects_an_uninhabited_public_position() {
    let mut vocabulary = SyntaxVocabulary::current();
    for entry in &mut vocabulary.entries {
        entry
            .uses
            .retain(|usage| usage.position != GrammarPosition::StaticDeclaration);
    }
    vocabulary.entries.retain(|entry| !entry.uses.is_empty());

    let error = vocabulary.validate().unwrap_err().to_string();
    assert!(error.contains("published grammar position `StaticDeclaration` has no syntax use"));
}

#[test]
fn validation_rejects_a_legacy_core_authoring_use() {
    let mut vocabulary = SyntaxVocabulary::current();
    let entry = vocabulary.lookup("true").unwrap().clone();
    let target = vocabulary
        .entries
        .iter_mut()
        .find(|candidate| candidate.spelling == entry.spelling)
        .unwrap();
    target.uses.push(SyntaxUse::new(
        VocabularyCategory::ReservedLiteral,
        GrammarPosition::CoreBooleanLiteral,
        CanonicalRole::BooleanLiteral,
    ));
    target.uses.sort_unstable();

    let error = vocabulary.validate().unwrap_err().to_string();
    assert!(error.contains("legacy core-authoring syntax"));
}

#[test]
fn validation_rejects_incomplete_or_incoherent_data() {
    let mut empty = SyntaxVocabulary::current();
    empty.entries.clear();
    assert_invalid(empty);

    let mut missing_use = SyntaxVocabulary::current();
    missing_use
        .entries
        .iter_mut()
        .find(|entry| entry.uses.len() > 1)
        .unwrap()
        .uses
        .pop();
    assert_invalid(missing_use);

    let mut wrong_layer = SyntaxVocabulary::current();
    wrong_layer.entries[0].uses[0].layer = LanguageLayer::StaticProgram;
    assert_invalid(wrong_layer);

    let mut wrong_category = SyntaxVocabulary::current();
    wrong_category.entries[0].uses[0].category = VocabularyCategory::BuiltinFunction;
    assert_invalid(wrong_category);

    let mut wrong_role = SyntaxVocabulary::current();
    wrong_role.entries[0].uses[0].canonical_role = CanonicalRole::PureFunction;
    assert_invalid(wrong_role);

    let mut wrong_policy = SyntaxVocabulary::current();
    wrong_policy.entries[0].identifier_policy = IdentifierPolicy::Allowed;
    assert_invalid(wrong_policy);

    let mut lexer_keyword = SyntaxVocabulary::current();
    lexer_keyword.lexer_keywords.push("project".into());
    assert_invalid(lexer_keyword);
}

#[test]
fn language_spec_validation_rejects_wrong_build_identity() {
    let mut wrong_version = language_spec();
    wrong_version.language_version = "0.0.0-invalid".into();
    assert!(wrong_version.validate().is_err());

    let mut wrong_schema = language_spec();
    wrong_schema.schema_version -= 1;
    assert!(wrong_schema.validate().is_err());
}

fn assert_invalid(value: SyntaxVocabulary) {
    assert!(value.validate().is_err());
}
