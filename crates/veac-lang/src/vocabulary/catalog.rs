use std::collections::BTreeMap;

use super::{
    all_control_uses, CanonicalRole, GrammarPosition, IdentifierPolicy, LanguageLayer,
    ReservedLiteral, SyntaxUse, SyntaxVocabulary, VocabularyCategory, VocabularyEntry,
};
use super::{
    catalog_core, catalog_expression, catalog_modifier_generator, catalog_output,
    catalog_structure, catalog_text_timeline, catalog_units, catalog_visual,
};
use crate::program::expression::{
    BuiltinFunction, CollectionOperation, PrimitiveType, TypeConstructor,
};

pub(super) struct TermSet {
    pub(super) context: GrammarPosition,
    pub(super) tokens: &'static [&'static str],
}

pub(super) const TERM_SET_GROUPS: &[&[TermSet]] = &[
    &catalog_core::KIND_SETS,
    &catalog_core::RESOURCE_SETS,
    &catalog_core::MEMBER_VALUE_SETS,
    &catalog_core::RELATION_ANNOTATION_SETS,
    &catalog_expression::SETS,
    &catalog_structure::SETS,
    &catalog_modifier_generator::SETS,
    &catalog_visual::SETS,
    &catalog_text_timeline::SETS,
    &catalog_output::SETS,
];

impl TermSet {
    pub(super) const fn new(context: GrammarPosition, tokens: &'static [&'static str]) -> Self {
        Self { context, tokens }
    }
}

pub(super) fn current_vocabulary() -> SyntaxVocabulary {
    assert_type_constructor_controls();
    let mut catalog = Catalog::default();
    for literal in ReservedLiteral::ALL {
        for position in [
            GrammarPosition::CoreBooleanLiteral,
            GrammarPosition::ExpressionBooleanLiteral,
        ] {
            catalog.add(
                literal.as_str(),
                IdentifierPolicy::Reserved,
                VocabularyCategory::ReservedLiteral,
                position,
                CanonicalRole::BooleanLiteral,
            );
        }
    }
    for usage in all_control_uses() {
        catalog.add(
            usage.as_str(),
            IdentifierPolicy::Allowed,
            VocabularyCategory::ContextualControl,
            usage.position(),
            usage.role(),
        );
    }
    for sets in TERM_SET_GROUPS {
        catalog.add_sets(sets);
    }
    for function in BuiltinFunction::ALL
        .iter()
        .map(|value| value.as_str())
        .chain(CollectionOperation::ALL.iter().map(|value| value.as_str()))
    {
        catalog.add(
            function,
            IdentifierPolicy::Allowed,
            VocabularyCategory::BuiltinFunction,
            GrammarPosition::ExpressionFunctionCallee,
            CanonicalRole::PureFunction,
        );
    }
    catalog_units::add(&mut catalog);
    for value_type in PrimitiveType::ALL {
        catalog.add(
            value_type.as_str(),
            IdentifierPolicy::Allowed,
            VocabularyCategory::ValueType,
            GrammarPosition::ValueTypePosition,
            CanonicalRole::CompileTimeType,
        );
    }
    catalog.finish()
}

fn assert_type_constructor_controls() {
    let controls = super::control_uses::structural_type::ALL;
    assert_eq!(TypeConstructor::ALL.len(), controls.len());
    for (constructor, control) in TypeConstructor::ALL.iter().zip(controls) {
        assert_eq!(constructor.as_str(), control.as_str());
        assert_eq!(
            control.position(),
            GrammarPosition::StructuralTypeConstructor
        );
        assert_eq!(control.role(), CanonicalRole::KindDiscriminator);
    }
}

#[derive(Default)]
pub(super) struct Catalog {
    entries: BTreeMap<String, VocabularyEntry>,
}

impl Catalog {
    fn add_sets(&mut self, sets: &[TermSet]) {
        for set in sets {
            for token in set.tokens {
                self.add(
                    token,
                    identifier_policy(token),
                    VocabularyCategory::EnumValue,
                    set.context,
                    CanonicalRole::ClosedValue,
                );
            }
        }
    }

    pub(super) fn add(
        &mut self,
        spelling: &str,
        policy: IdentifierPolicy,
        category: VocabularyCategory,
        position: GrammarPosition,
        role: CanonicalRole,
    ) {
        if position.layer() == LanguageLayer::CoreAuthoring {
            return;
        }
        let entry = self
            .entries
            .entry(spelling.to_owned())
            .or_insert_with(|| VocabularyEntry {
                spelling: spelling.to_owned(),
                identifier_policy: policy,
                uses: Vec::new(),
            });
        assert_eq!(entry.identifier_policy, policy, "policy for {spelling}");
        entry.uses.push(SyntaxUse::new(category, position, role));
    }

    fn finish(self) -> SyntaxVocabulary {
        let entries = self
            .entries
            .into_values()
            .map(|mut entry| {
                entry.uses.sort_unstable();
                entry.uses.dedup();
                entry
            })
            .collect();
        SyntaxVocabulary {
            lexer_keywords: Vec::new(),
            entries,
        }
    }
}

pub(super) fn identifier_policy(spelling: &str) -> IdentifierPolicy {
    if crate::name::has_name_shape(spelling) {
        IdentifierPolicy::Allowed
    } else {
        IdentifierPolicy::NotApplicable
    }
}
