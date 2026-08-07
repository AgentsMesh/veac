use super::super::{
    control_uses::expression as controls, language_spec, CanonicalRole, GrammarPosition,
    LanguageLayer, VocabularyCategory,
};
use crate::program::expression::FunctionEffect;
use crate::SyntaxToken;

#[test]
fn function_effect_syntax_has_typed_sources_of_truth() {
    assert_eq!(controls::FUNCTION_EFFECT.as_str(), "effect");
    assert_eq!(
        controls::FUNCTION_EFFECT.position(),
        GrammarPosition::ExpressionFunctionEffectClause
    );
    assert_eq!(
        controls::FUNCTION_EFFECT.role(),
        CanonicalRole::ClauseIntroducer
    );
    assert_eq!(
        controls::FUNCTION_EFFECT.layer(),
        LanguageLayer::ExecutableExpression
    );
    for effect in FunctionEffect::ALL {
        assert_eq!(FunctionEffect::parse(effect.as_str()), Some(*effect));
        assert_eq!(
            <FunctionEffect as SyntaxToken>::parse(effect.as_str()),
            Some(*effect)
        );
    }
}

#[test]
fn language_spec_publishes_effect_values_in_the_executable_layer() {
    let vocabulary = language_spec().vocabulary;
    for effect in FunctionEffect::ALL {
        let usage = vocabulary
            .lookup(effect.as_str())
            .unwrap()
            .uses
            .iter()
            .find(|usage| usage.position == GrammarPosition::ExpressionFunctionEffectValue)
            .unwrap();
        assert_eq!(usage.category, VocabularyCategory::EnumValue);
        assert_eq!(usage.layer, LanguageLayer::ExecutableExpression);
        assert_eq!(usage.canonical_role, CanonicalRole::ClosedValue);
    }
}
