use std::collections::BTreeSet;

use super::{
    all_control_uses, is_reserved_literal, language_spec, CanonicalRole, ControlWord,
    GrammarPosition, LanguageSpec, ReservedLiteral, SyntaxUse, SyntaxVocabulary, UnitSuffix,
    VocabularyCategory, VocabularyEntry, LANGUAGE_SPEC_SCHEMA_VERSION,
};

mod delivery_control_sets;
mod domain_contracts;
mod domain_validation_branches;
mod enum_sets;
mod expression_names;
mod function_effects;
mod legacy_surface;
mod member_value_sets;
mod modifier_generator_sets;
mod nominal_method_controls;
mod output_delivery_sets;
mod owner_control_sets;
mod plugin_effects;
mod plugin_validation_branches;
mod plural_owned_methods;
mod schema;
mod standard_library_projection;
mod standard_library_validation_branches;
mod structure_sets;
mod surface_statistics;
mod text_timeline_sets;
mod validation_contract;
mod visual_sets;

#[test]
fn controls_are_unique_spellings_with_closed_grammar_uses() {
    let vocabulary = language_spec().vocabulary;
    let words = ControlWord::all();
    let spellings = words
        .iter()
        .copied()
        .map(ControlWord::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(spellings.len(), words.len());

    let uses = all_control_uses()
        .filter(|usage| usage.layer().is_public())
        .collect::<Vec<_>>();
    assert_eq!(
        uses.iter().copied().collect::<BTreeSet<_>>().len(),
        uses.len()
    );
    for word in words {
        assert_eq!(ControlWord::parse(word.as_str()), Some(word));
        let entry = vocabulary.lookup(word.as_str()).unwrap();
        assert!(entry
            .uses
            .iter()
            .any(|usage| usage.category == VocabularyCategory::ContextualControl));
    }
    assert_eq!(ControlWord::parse("unknown"), None);
}

#[test]
fn runtime_control_word_construction_preserves_the_spelling() {
    assert_eq!(
        ControlWord::new("runtime-control").as_str(),
        "runtime-control"
    );
}

#[test]
fn removed_macro_controls_do_not_leak_into_the_executable_surface() {
    let vocabulary = language_spec().vocabulary;
    for spelling in [
        "project",
        "preset",
        "component",
        "instance",
        "default",
        "bind",
    ] {
        assert!(vocabulary.lookup(spelling).is_none(), "{spelling}");
        assert!(ControlWord::parse(spelling).is_none(), "{spelling}");
    }
}

#[test]
fn function_and_structural_type_controls_keep_distinct_uses() {
    let vocabulary = language_spec().vocabulary;
    let function = vocabulary.lookup("fn").unwrap();
    assert_use(
        function,
        GrammarPosition::StaticDeclaration,
        CanonicalRole::DeclarationIntroducer,
    );
    assert_use(
        function,
        GrammarPosition::ExpressionClosureIntroducer,
        CanonicalRole::ClauseIntroducer,
    );
    for constructor in ["list", "map", "range", "fn"] {
        let entry = vocabulary.lookup(constructor).unwrap();
        assert_use(
            entry,
            GrammarPosition::StructuralTypeConstructor,
            CanonicalRole::KindDiscriminator,
        );
        assert!(!entry
            .uses
            .iter()
            .any(|usage| usage.position == GrammarPosition::ValueTypePosition));
    }
    assert_use(
        vocabulary.lookup("by").unwrap(),
        GrammarPosition::ExpressionRangeStep,
        CanonicalRole::ClauseIntroducer,
    );
    assert_eq!(ControlWord::parse("fn").unwrap().as_str(), "fn");
    assert!(vocabulary.lookup(":").is_none());
    assert!(vocabulary.lookup("->").is_none());
}

#[test]
fn catalog_has_one_entry_per_spelling_and_all_categories() {
    let vocabulary = language_spec().vocabulary;
    vocabulary.validate().unwrap();
    let spellings = vocabulary
        .entries
        .iter()
        .map(|entry| entry.spelling.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(spellings.len(), vocabulary.entries.len());
    assert!(vocabulary.lexer_keywords.is_empty());
    for category in VocabularyCategory::ALL {
        assert!(vocabulary.count(category) > 0, "{category:?}");
        assert_eq!(
            vocabulary.count(category),
            vocabulary.in_category(category).count()
        );
    }
    let positions = vocabulary
        .entries
        .iter()
        .flat_map(|entry| &entry.uses)
        .map(|usage| usage.position)
        .collect::<BTreeSet<_>>();
    assert_eq!(positions, GrammarPosition::ALL.iter().copied().collect());
    assert!(vocabulary.entries.iter().all(|entry| entry
        .uses
        .iter()
        .all(|usage| usage.layer.is_public() && usage.position.is_public())));
}

#[test]
fn literals_and_units_have_typed_sources_of_truth() {
    assert!(is_reserved_literal("true"));
    assert!(is_reserved_literal("false"));
    assert!(!is_reserved_literal("project"));
    for literal in ReservedLiteral::ALL {
        assert_eq!(ReservedLiteral::parse(literal.as_str()), Some(literal));
    }
    for unit in UnitSuffix::ALL {
        assert_eq!(UnitSuffix::parse(unit.as_str()), Some(unit));
    }
}

#[test]
fn language_spec_v7_is_deterministic_and_round_trips() {
    let current = language_spec();
    assert_eq!(current.schema_version, LANGUAGE_SPEC_SCHEMA_VERSION);
    assert_eq!(current.schema_version, 7);
    let first = serde_json::to_string_pretty(&current).unwrap();
    assert_eq!(
        first,
        serde_json::to_string_pretty(&language_spec()).unwrap()
    );
    let decoded: LanguageSpec = serde_json::from_str(&first).unwrap();
    decoded.validate().unwrap();
    assert_eq!(decoded, current);
}

#[test]
fn default_specs_are_the_current_closed_contracts() {
    assert_eq!(LanguageSpec::default(), LanguageSpec::current());
    assert_eq!(SyntaxVocabulary::default(), SyntaxVocabulary::current());
}

fn assert_use(entry: &VocabularyEntry, position: GrammarPosition, role: CanonicalRole) {
    assert!(entry.uses.contains(&SyntaxUse {
        category: VocabularyCategory::ContextualControl,
        layer: position.layer(),
        position,
        canonical_role: role,
    }));
}
