use std::collections::BTreeSet;

use super::support::*;
use veac_lang::vocabulary::{GrammarPosition, IdentifierPolicy, LanguageSpec, VocabularyCategory};

#[path = "language_spec/plugin_effects.rs"]
mod plugin_effects;

#[test]
fn language_spec_help_and_dispatch_are_public() {
    veac()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("language-spec"));
    veac()
        .args(["language-spec", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--schema"));
}

#[test]
fn language_spec_stdout_is_canonical_deterministic_json() {
    let first = veac().arg("language-spec").output().unwrap();
    let second = veac().arg("language-spec").output().unwrap();
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let value = assert_canonical_json(&first.stdout);
    assert_eq!(
        value["schema_version"],
        veac_lang::vocabulary::LANGUAGE_SPEC_SCHEMA_VERSION
    );
    assert_eq!(value["language"], "veac");
    assert_eq!(value["language_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["vocabulary"]["lexer_keywords"], serde_json::json!([]));
    assert!(value["vocabulary"]["entries"][0]["uses"].is_array());
    assert!(!value["vocabulary"].to_string().contains("core_authoring"));
    assert_eq!(
        value["domain_opset"]["types"].as_array().unwrap().len(),
        214
    );
    assert_eq!(
        value["domain_opset"]["operations"]
            .as_array()
            .unwrap()
            .len(),
        581
    );
    assert_eq!(
        value["standard_library"]["free_functions"]
            .as_array()
            .unwrap()
            .len(),
        558
    );
    assert_eq!(
        value["standard_library"]["methods"]
            .as_array()
            .unwrap()
            .len(),
        23
    );
    let registry = veac_lang::program::DomainOperationRegistry::standard();
    assert_eq!(value["domain_opset"]["version"], registry.version().raw());
    assert_eq!(
        value["domain_opset"]["registry_digest"],
        registry.digest().to_string()
    );
}

#[test]
fn language_spec_schema_flag_matches_the_schema_registry() {
    let direct = veac().args(["language-spec", "--schema"]).output().unwrap();
    let registry = veac()
        .args(["schema", "--contract", "language-spec"])
        .output()
        .unwrap();
    assert!(direct.status.success());
    assert_eq!(direct.stdout, registry.stdout);
    let schema = assert_canonical_json(&direct.stdout);
    assert!(schema["properties"]["vocabulary"].is_object());
    assert_eq!(
        schema["properties"]["schema_version"]["const"],
        veac_lang::vocabulary::LANGUAGE_SPEC_SCHEMA_VERSION
    );
    assert_eq!(
        schema["$defs"]["SyntaxVocabulary"]["properties"]["lexer_keywords"]["maxItems"],
        0
    );
    assert_eq!(
        schema["$defs"]["SyntaxVocabulary"]["properties"]["entries"]["uniqueItems"],
        true
    );
    assert_eq!(
        schema["$defs"]["VocabularyEntry"]["properties"]["uses"]["uniqueItems"],
        true
    );
    assert_eq!(
        schema["$defs"]["DomainOpsetSpec"]["properties"]["operations"]["uniqueItems"],
        true
    );
    assert_eq!(
        schema["$defs"]["StandardLibrarySpec"]["additionalProperties"],
        false
    );
    assert!(!schema.to_string().contains("core_authoring"));
}

#[test]
fn language_spec_stdout_has_complete_sorted_vocabulary() {
    let output = veac().arg("language-spec").output().unwrap();
    assert!(output.status.success());
    let spec: LanguageSpec = serde_json::from_slice(&output.stdout).unwrap();
    spec.validate().unwrap();
    assert_eq!(
        spec.schema_version,
        veac_lang::vocabulary::LANGUAGE_SPEC_SCHEMA_VERSION
    );
    assert!(spec.vocabulary.lexer_keywords.is_empty());

    let entries = &spec.vocabulary.entries;
    assert!(entries
        .windows(2)
        .all(|pair| pair[0].spelling < pair[1].spelling));
    assert!(entries
        .iter()
        .all(|entry| entry.uses.windows(2).all(|pair| pair[0] < pair[1])));

    let reserved = spec
        .vocabulary
        .in_category(VocabularyCategory::ReservedLiteral)
        .map(|entry| {
            assert_eq!(entry.identifier_policy, IdentifierPolicy::Reserved);
            entry.spelling.as_str()
        })
        .collect::<BTreeSet<_>>();
    let expected = ["false", "true"].into_iter().collect::<BTreeSet<_>>();
    assert_eq!(reserved, expected);

    let inhabited = entries
        .iter()
        .flat_map(|entry| entry.uses.iter().map(|usage| usage.position))
        .collect::<BTreeSet<_>>();
    let positions = GrammarPosition::ALL
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(inhabited, positions);
    assert!(entries.iter().all(|entry| entry
        .uses
        .iter()
        .all(|usage| usage.layer.is_public() && usage.position.is_public())));
}

#[test]
fn language_spec_stdout_maps_every_domain_opcode_without_adding_keywords() {
    let output = veac().arg("language-spec").output().unwrap();
    assert!(output.status.success());
    let spec: LanguageSpec = serde_json::from_slice(&output.stdout).unwrap();
    let operations = spec
        .domain_opset
        .operations
        .iter()
        .map(|operation| operation.opcode)
        .collect::<BTreeSet<_>>();
    let mappings = spec
        .standard_library
        .free_functions
        .iter()
        .map(|function| function.operation_opcode)
        .chain(
            spec.standard_library
                .methods
                .iter()
                .map(|method| method.operation_opcode),
        )
        .collect::<BTreeSet<_>>();
    assert_eq!(operations, mappings);
    assert_eq!(operations.len(), 581);
    assert_eq!(spec.vocabulary.entries.len(), 91);
    assert!(spec.vocabulary.lexer_keywords.is_empty());
}

fn assert_canonical_json(bytes: &[u8]) -> serde_json::Value {
    assert!(bytes.ends_with(b"\n"));
    assert!(!bytes.ends_with(b"\n\n"));
    let body = &bytes[..bytes.len() - 1];
    let value = serde_json::from_slice(body).unwrap();
    assert_eq!(body, serde_json_canonicalizer::to_vec(&value).unwrap());
    value
}
