use super::support::*;
use veac_lang::program::{
    SourceIndexBuildInputType, SourceIndexInventory, SOURCE_INDEX_SCHEMA,
    SOURCE_INDEX_SCHEMA_VERSION,
};
use veac_lang::source_edit::{BodySite, ExpressionSite, SourceNodeRef, SourceRevision};

fn indexed_source() -> String {
    format!(
        "const time duration = 100ms + 100ms;\n\
         fn twice(value: time) -> time {{ value + value }}\n{}",
        EXECUTABLE_SOURCE
    )
}

fn output(source: &std::path::Path) -> std::process::Output {
    veac()
        .args(["source-index", source.to_str().unwrap()])
        .output()
        .unwrap()
}

#[test]
fn source_index_is_deterministic_and_discovers_executable_edit_sites() {
    let temp = tempdir().unwrap();
    let source_text = indexed_source();
    let source = source_file(&temp, &source_text);
    let first = output(&source);
    let second = output(&source);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
    let inventory: SourceIndexInventory = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(inventory.schema, SOURCE_INDEX_SCHEMA);
    assert_eq!(inventory.schema_version, SOURCE_INDEX_SCHEMA_VERSION);
    assert!(inventory
        .nodes
        .windows(2)
        .all(|pair| pair[0].target < pair[1].target));
    assert_expression(
        &inventory,
        SourceNodeRef::constant("main.veac", "duration"),
        ExpressionSite::ConstantValue,
        "100ms + 100ms",
        &source_text,
    );
    let body = node(&inventory, SourceNodeRef::function("main.veac", "twice"))
        .bodies
        .iter()
        .find(|value| value.site == BodySite::FunctionBody)
        .unwrap();
    assert_eq!(body.source, "{ value + value }");
    assert_eq!(&source_text[body.range.start..body.range.end], body.source);
}

#[test]
fn source_revision_and_index_share_one_public_contract() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &indexed_source());
    let inventory: SourceIndexInventory = serde_json::from_slice(&output(&source).stdout).unwrap();
    let revision = veac()
        .args(["source-revision", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(
        serde_json::from_slice::<SourceRevision>(&revision.stdout).unwrap(),
        inventory.revision
    );
    veac()
        .args(["schema", "--contract", "source-index"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SourceIndexInventory"));
}

#[test]
fn source_index_discovers_enum_inputs_without_runtime_bindings() {
    let temp = tempdir().unwrap();
    let source = source_file(
        &temp,
        &format!(
            "enum Locale {{ en, pt-BR, zh-Hans, }}\n\
             input parameter locale: Locale;\n{EXECUTABLE_SOURCE}"
        ),
    );
    let inventory: SourceIndexInventory = serde_json::from_slice(&output(&source).stdout).unwrap();
    let [input] = inventory.build_inputs.as_slice() else {
        panic!("expected one discoverable Build input")
    };
    assert_eq!(input.name, "locale");
    assert_eq!(input.role, veac_lang::program::BuildInputRole::Parameter);
    let SourceIndexBuildInputType::Enum {
        name,
        type_id,
        definition_sha256,
        variants,
    } = &input.value_type
    else {
        panic!("locale input must expose its payloadless enum interface")
    };
    assert_eq!(name, "Locale");
    assert_eq!(type_id.len(), 64);
    assert_eq!(definition_sha256.len(), 64);
    assert_eq!(variants, &["en", "pt-BR", "zh-Hans"]);
}

fn node(
    inventory: &SourceIndexInventory,
    target: SourceNodeRef,
) -> &veac_lang::program::SourceIndexNode {
    inventory
        .nodes
        .iter()
        .find(|value| value.target == target)
        .unwrap()
}

fn assert_expression(
    inventory: &SourceIndexInventory,
    target: SourceNodeRef,
    site: ExpressionSite,
    expected: &str,
    source: &str,
) {
    let expression = node(inventory, target)
        .expressions
        .iter()
        .find(|value| value.site == site)
        .unwrap();
    assert_eq!(expression.source, expected);
    assert_eq!(
        &source[expression.range.start..expression.range.end],
        expected
    );
}
