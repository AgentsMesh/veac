use super::support::*;
use veac_lang::program::{SourceIndexInventory, SOURCE_INDEX_SCHEMA, SOURCE_INDEX_SCHEMA_VERSION};
use veac_lang::source_edit::{ExpressionSite, SourceNodeRef, SourceRevision};

fn indexed_source() -> String {
    format!(
        r#"component sequence card {{
  param time duration default 100ms + 100ms;
  body {{}}
}}
instance sequence card-one from card {{ bind duration 300ms; }}
{GENERATED_SOURCE}"#
    )
}

fn output(source: &std::path::Path) -> std::process::Output {
    veac()
        .args(["source-index", source.to_str().unwrap()])
        .output()
        .unwrap()
}

#[test]
fn source_index_is_deterministic_and_discovers_every_editable_site_kind() {
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
    assert!(
        node(&inventory, SourceNodeRef::project("main.veac", "cli-e2e"))
            .expressions
            .is_empty()
    );
    assert_site(
        &inventory,
        &source_text,
        SourceNodeRef::component("main.veac", "card"),
        ExpressionSite::ComponentParameterDefault {
            parameter: "duration".into(),
        },
        "100ms + 100ms",
    );
    assert_site(
        &inventory,
        &source_text,
        SourceNodeRef::component_instance("main.veac", "card-one"),
        ExpressionSite::ComponentInstanceArgument {
            parameter: "duration".into(),
        },
        "300ms",
    );
    assert_site(
        &inventory,
        &source_text,
        SourceNodeRef::item("main.veac", "cli-e2e", "main", "base", "background"),
        ExpressionSite::ItemRecordDuration,
        "200ms",
    );
}

#[test]
fn source_index_revision_and_schema_match_the_public_contracts() {
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
    veac()
        .args(["source-index", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("agent-readable inventory"));
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

fn assert_site(
    inventory: &SourceIndexInventory,
    source: &str,
    target: SourceNodeRef,
    site: ExpressionSite,
    expected: &str,
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
