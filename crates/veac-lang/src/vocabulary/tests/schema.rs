use serde_json::{json, Value};

use super::super::{
    language_spec_json_schema, DomainOperationSignature, GrammarPosition, LanguageLayer,
    LanguageSpec, LANGUAGE_SPEC_SCHEMA, LANGUAGE_SPEC_SCHEMA_VERSION,
};

#[test]
fn schema_locks_the_current_build_identity() {
    let schema = language_spec_json_schema().unwrap();
    for (field, expected) in [
        ("schema", json!(LANGUAGE_SPEC_SCHEMA)),
        ("schema_version", json!(LANGUAGE_SPEC_SCHEMA_VERSION)),
        ("language", json!("veac")),
        ("language_version", json!(env!("CARGO_PKG_VERSION"))),
    ] {
        assert_eq!(property(&schema, field).get("const"), Some(&expected));
    }
}

#[test]
fn schema_requires_an_empty_lexer_keyword_table() {
    let schema = language_spec_json_schema().unwrap();
    let vocabulary = definition(&schema, "SyntaxVocabulary");
    assert_eq!(
        property(vocabulary, "lexer_keywords").get("maxItems"),
        Some(&json!(0))
    );
}

#[test]
fn schema_publishes_expressible_registry_shape_invariants() {
    let schema = language_spec_json_schema().unwrap();
    let vocabulary = definition(&schema, "SyntaxVocabulary");
    assert_nonempty_unique(property(vocabulary, "entries"));

    let entry = definition(&schema, "VocabularyEntry");
    assert_eq!(
        property(entry, "spelling").get("minLength"),
        Some(&json!(1))
    );
    assert_eq!(
        property(entry, "spelling").get("pattern"),
        Some(&json!(r"^\S+$"))
    );
    assert_nonempty_unique(property(entry, "uses"));
}

#[test]
fn schema_exposes_every_v7_language_dimension() {
    let schema = language_spec_json_schema().unwrap();
    for name in [
        "LanguageSpec",
        "lexer_keywords",
        "identifier_policy",
        "GrammarPosition",
        "LanguageLayer",
        "canonical_role",
        "standard_library",
        "domain_opset",
        "plugin_effects",
        "PluginEffectSpec",
        "PluginParameterSpec",
        "DomainOperationSignature",
        "DomainMaxStageSpec",
        "TemporalLoweringOpcodeSpec",
        "DomainValueShapeSpec",
        "StandardLibraryMethod",
    ] {
        assert!(
            schema.to_string().contains(name),
            "schema is missing {name}"
        );
    }
}

#[test]
fn schema_publishes_strict_domain_and_standard_library_arrays() {
    let schema = language_spec_json_schema().unwrap();
    for (definition_name, field) in [
        ("DomainOpsetSpec", "types"),
        ("DomainOpsetSpec", "operations"),
        ("StandardLibrarySpec", "types"),
        ("StandardLibrarySpec", "free_functions"),
        ("StandardLibrarySpec", "methods"),
        ("PluginEffectSpec", "parameters"),
        ("PluginEffectSpec", "supported_backends"),
    ] {
        let definition = definition(&schema, definition_name);
        assert_nonempty_unique(property(definition, field));
        assert_eq!(definition.get("additionalProperties"), Some(&json!(false)));
    }
    assert_eq!(
        property(definition(&schema, "DomainOpsetSpec"), "registry_digest").get("pattern"),
        Some(&json!(r"^[0-9a-f]{64}$"))
    );
}

#[test]
fn schema_closes_the_temporal_domain_availability_contract() {
    let schema = language_spec_json_schema().unwrap();
    let signature = definition(&schema, "DomainOperationSignature");
    let required = signature.get("required").and_then(Value::as_array).unwrap();
    assert!(required.contains(&json!("max_stage")));
    assert!(property(signature, "temporal_lowering").is_object());
    assert_eq!(signature.get("additionalProperties"), Some(&json!(false)));
    assert_eq!(
        definition(&schema, "DomainMaxStageSpec").get("enum"),
        Some(&json!(["build", "temporal"]))
    );
    assert_eq!(
        definition(&schema, "TemporalLoweringOpcodeSpec").get("enum"),
        Some(&json!(["compose_vector", "compose_point", "compose_rect"]))
    );

    let mut value =
        serde_json::to_value(&LanguageSpec::current().domain_opset.operations[0].contract).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("unknown".to_owned(), json!(true));
    assert!(serde_json::from_value::<DomainOperationSignature>(value).is_err());
}

#[test]
fn schema_positions_match_the_complete_runtime_inventory() {
    let schema = language_spec_json_schema().unwrap();
    let published = definition(&schema, "GrammarPosition")
        .get("enum")
        .and_then(Value::as_array)
        .unwrap();
    let runtime = GrammarPosition::ALL
        .iter()
        .map(|position| serde_json::to_value(position).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(published, &runtime);
}

#[test]
fn schema_publishes_only_reachable_surface_layers() {
    let schema = language_spec_json_schema().unwrap();
    let published = definition(&schema, "LanguageLayer").get("enum").unwrap();
    assert_eq!(
        published,
        &json!(["static_program", "executable_expression"])
    );
    assert!(!schema.to_string().contains("core_authoring"));
    assert!(!schema.to_string().contains("settings_member"));
    assert_eq!(
        serde_json::from_value::<LanguageLayer>(json!("executable_expression")).unwrap(),
        LanguageLayer::ExecutableExpression
    );
    assert!(serde_json::from_value::<LanguageLayer>(json!("pure_expression")).is_err());
}

fn property<'a>(schema: &'a Value, name: &str) -> &'a Value {
    schema
        .get("properties")
        .and_then(|value| value.get(name))
        .unwrap_or_else(|| panic!("schema property {name} is missing"))
}

fn definition<'a>(schema: &'a Value, name: &str) -> &'a Value {
    schema
        .get("$defs")
        .and_then(|value| value.get(name))
        .unwrap_or_else(|| panic!("schema definition {name} is missing"))
}

fn assert_nonempty_unique(schema: &Value) {
    assert_eq!(schema.get("minItems"), Some(&json!(1)));
    assert_eq!(schema.get("uniqueItems"), Some(&json!(true)));
}
