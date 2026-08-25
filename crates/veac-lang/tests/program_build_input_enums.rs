use std::fs;

use tempfile::tempdir;
use veac_lang::program::{
    parse_build_input_manifest, prepare_path, prepare_source, BuildInputBinding,
    BuildInputManifestV1, BuildInputManifestValue, SourceIndexBuildInputType,
};

#[path = "program_functions/support.rs"]
mod support;

#[test]
fn root_payloadless_enum_drives_execution_and_variant_sensitive_identity() {
    let prepared = prepare_source(&root_source()).unwrap();
    let zh = prepared.execute_with_inputs(&manifest("zh-Hans")).unwrap();
    let same = prepared.execute_with_inputs(&manifest("zh-Hans")).unwrap();
    let en = prepared.execute_with_inputs(&manifest("en")).unwrap();
    assert_eq!(support::result_duration(&zh), "1s");
    assert_eq!(support::result_duration(&en), "2s");
    assert_eq!(identity(&zh), identity(&same));
    assert_ne!(identity(&zh), identity(&en));
}

#[test]
fn imported_payloadless_enum_resolves_by_its_nominal_declaration() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("locales.veac"),
        "module { export enum Locale { zh-Hans, en, ja, } }",
    )
    .unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, imported_source()).unwrap();
    let prepared = prepare_path(&entry).unwrap();
    let declaration = &prepared.build_input_declarations()["locale"];
    assert_eq!(declaration.value_type().to_string(), "localization.Locale");
    let inventory = prepared.source_inventory().unwrap();
    let [input] = inventory.build_inputs.as_slice() else {
        panic!("expected the imported enum Build input")
    };
    let SourceIndexBuildInputType::Enum { name, variants, .. } = &input.value_type else {
        panic!("locale must be indexed as an enum")
    };
    assert_eq!(name, "localization.Locale");
    assert_eq!(variants, &["zh-Hans", "en", "ja"]);
    let built = prepared.execute_with_inputs(&manifest("ja")).unwrap();
    assert_eq!(support::result_duration(&built), "3s");
}

#[test]
fn enum_definition_changes_input_identity_even_when_selected_variant_is_unchanged() {
    let first = prepare_source(&root_source()).unwrap();
    let changed = prepare_source(&root_source().replace(
        "enum Locale { zh-Hans, en, ja, }",
        "enum Locale { en, zh-Hans, ja, }",
    ))
    .unwrap();
    let first = first.execute_with_inputs(&manifest("ja")).unwrap();
    let changed = changed.execute_with_inputs(&manifest("ja")).unwrap();
    assert_ne!(identity(&first), identity(&changed));
}

#[test]
fn enum_contract_rejects_unknown_variants_wrong_tags_and_non_leaf_types() {
    let prepared = prepare_source(&root_source()).unwrap();
    assert_execute_error(
        &prepared,
        &manifest("Unknown"),
        "PROGRAM_INPUT_ENUM_VARIANT",
    );
    let mut primitive = manifest("zh-Hans");
    primitive.inputs[0].value = BuildInputManifestValue::Text {
        value: "zh-Hans".into(),
    };
    assert_execute_error(&prepared, &primitive, "PROGRAM_INPUT_TYPE_MISMATCH");

    for source in [
        "struct Locale {} input parameter locale: Locale;",
        "enum Locale { Regional { region: text, }, } input parameter locale: Locale;",
    ] {
        let error = prepare_source(&support::project_with(source, "1s")).unwrap_err();
        assert_eq!(error.as_slice()[0].code, "PROGRAM_INPUT_TYPE");
    }
}

#[test]
fn enum_manifest_tag_is_strictly_decoded_in_v1_and_denies_unknown_fields() {
    let json = r#"{
      "schema":"https://veac.dev/schemas/build-inputs",
      "schema_version":1,
      "inputs":[{"name":"locale","value":{"type":"enum","value":"zh-Hans"}}]
    }"#;
    let parsed = parse_build_input_manifest(json).unwrap();
    assert_eq!(parsed, manifest("zh-Hans"));
    let unknown = json.replace(
        "\"value\":\"zh-Hans\"",
        "\"value\":\"zh-Hans\",\"payload\":null",
    );
    assert_eq!(
        parse_build_input_manifest(&unknown).unwrap_err().code(),
        "PROGRAM_INPUT_MANIFEST_JSON"
    );
}

fn root_source() -> String {
    support::project_with(
        r#"enum Locale { zh-Hans, en, ja, }
input parameter locale: Locale;
fn localized_duration() -> time {
  match locale {
    Locale.zh-Hans => 1s,
    Locale.en => 2s,
    Locale.ja => 3s,
  }
}"#,
        "localized_duration()",
    )
}

fn imported_source() -> String {
    support::project_with(
        r#"import "./locales.veac" as localization;
input parameter locale: localization.Locale;
fn localized_duration() -> time {
  match locale {
    localization.Locale.zh-Hans => 1s,
    localization.Locale.en => 2s,
    localization.Locale.ja => 3s,
  }
}"#,
        "localized_duration()",
    )
}

fn manifest(locale: &str) -> BuildInputManifestV1 {
    let mut manifest = BuildInputManifestV1::empty();
    manifest.inputs.push(BuildInputBinding {
        name: "locale".into(),
        value: BuildInputManifestValue::Enum {
            value: locale.into(),
        },
    });
    manifest
}

fn identity(program: &veac_lang::program::BuiltProgram) -> &str {
    &program.envelope().executable.digests.declared_inputs_sha256
}

fn assert_execute_error(
    prepared: &veac_lang::program::ExecutableBuild,
    manifest: &BuildInputManifestV1,
    code: &str,
) {
    assert_eq!(
        prepared
            .execute_with_inputs(manifest)
            .unwrap_err()
            .as_slice()[0]
            .code,
        code
    );
}
