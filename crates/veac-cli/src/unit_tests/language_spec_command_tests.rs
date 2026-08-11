use crate::{SchemaContract, SchemaFormat};

#[test]
fn language_spec_encoding_is_canonical_and_deterministic() {
    let first = crate::commands::language_spec_json(false).unwrap();
    let second = crate::commands::language_spec_json(false).unwrap();
    assert_eq!(first, second);
    let value = assert_canonical_json(&first);
    assert_eq!(value["schema"], "https://veac.dev/schemas/language-spec");
    assert_eq!(
        value["schema_version"],
        veac_lang::vocabulary::LANGUAGE_SPEC_SCHEMA_VERSION
    );
    assert_eq!(value["language"], "veac");
    assert_eq!(value["language_version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["plugin_effects"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["plugin_effects"][0]["effect_type"],
        veac_ir::REFERENCE_MONOCHROME_EFFECT_TYPE
    );
    assert_eq!(value["vocabulary"]["lexer_keywords"], serde_json::json!([]));
    assert!(value["vocabulary"]["entries"].is_array());
    let encoded = value["vocabulary"]["entries"].to_string();
    assert!(encoded.contains("\"executable_expression\""));
    assert!(!encoded.contains("\"core_authoring\""));
    assert!(!encoded.contains("\"pure_expression\""));
    assert_eq!(
        value["domain_opset"]["operations"]
            .as_array()
            .unwrap()
            .len(),
        582
    );
    assert_eq!(
        value["standard_library"]["free_functions"]
            .as_array()
            .unwrap()
            .len(),
        559
    );
    assert_eq!(
        value["standard_library"]["methods"]
            .as_array()
            .unwrap()
            .len(),
        23
    );
    let registry = veac_lang::program::DomainOperationRegistry::standard();
    assert_eq!(
        value["domain_opset"]["version"],
        serde_json::json!(registry.version().raw())
    );
    assert_eq!(
        value["domain_opset"]["registry_digest"],
        registry.digest().to_string()
    );
}

#[test]
fn language_spec_schema_mode_reuses_the_public_schema_contract() {
    let direct = crate::commands::language_spec_json(true).unwrap();
    let registry =
        crate::commands::schema_json(SchemaContract::LanguageSpec, SchemaFormat::JsonSchema)
            .unwrap();
    assert_eq!(direct, registry);
    let schema = assert_canonical_json(&direct);
    assert!(schema["properties"]["vocabulary"].is_object());
    assert_eq!(
        schema["properties"]["schema_version"]["const"],
        veac_lang::vocabulary::LANGUAGE_SPEC_SCHEMA_VERSION
    );
    assert!(schema["properties"]["domain_opset"].is_object());
    assert!(schema["properties"]["standard_library"].is_object());
    assert!(schema["properties"]["plugin_effects"].is_object());
    assert!(!schema.to_string().contains("core_authoring"));
}

fn assert_canonical_json(document: &str) -> serde_json::Value {
    assert!(document.ends_with('\n'));
    assert!(!document.ends_with("\n\n"));
    let body = document.strip_suffix('\n').unwrap();
    let value = serde_json::from_str(body).unwrap();
    assert_eq!(body, serde_json_canonicalizer::to_string(&value).unwrap());
    value
}
