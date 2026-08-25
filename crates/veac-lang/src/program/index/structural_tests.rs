use std::collections::BTreeMap;

use crate::source_edit::{SourceImportRef, SourceNodeRef, SourceTemporalProperty};

use super::SourceIndex;

const ENTRY: &str = r#"import "./brand.veac" as brand;
input parameter duration: time;
const time base = 1s;
struct Card { value: time, }
enum Mood { Calm, }
impl Card @timing { fn timing(self) -> time { self.value } }
animate visual-opacity on clip(@demo, @main, @visual, @first) { progress }
fn main(context: Context) -> Project { context.empty_project() }"#;

#[test]
fn v8_inventory_publishes_modules_imports_and_all_top_level_kinds() {
    let sources = BTreeMap::from([
        (
            "brand.veac".to_owned(),
            "module { export fn value() -> time { 2s } }".to_owned(),
        ),
        ("main.veac".to_owned(), ENTRY.to_owned()),
    ]);
    let index = super::test_support::index(&sources);
    let revision = super::test_revision(&index);
    let inventory = index.inventory(&revision).unwrap();
    assert_eq!(inventory.schema_version, super::SOURCE_INDEX_SCHEMA_VERSION);
    assert_eq!(
        inventory
            .modules
            .iter()
            .map(|value| value.module.as_str())
            .collect::<Vec<_>>(),
        ["brand.veac", "main.veac"]
    );
    let main = inventory
        .modules
        .iter()
        .find(|value| value.module == "main.veac")
        .unwrap();
    let imported = &main.imports[0];
    assert_eq!(imported.target, SourceImportRef::new("main.veac", "brand"));
    assert_eq!(imported.path, "./brand.veac");
    assert_eq!(
        &ENTRY[imported.range.start..imported.range.end],
        imported.source
    );
    for target in [
        SourceNodeRef::input("main.veac", "duration"),
        SourceNodeRef::constant("main.veac", "base"),
        SourceNodeRef::structure("main.veac", "Card"),
        SourceNodeRef::enumeration("main.veac", "Mood"),
        SourceNodeRef::implementation("main.veac", "Card", "timing"),
        SourceNodeRef::temporal(
            "main.veac",
            "demo",
            "main",
            "visual",
            "first",
            SourceTemporalProperty::VisualOpacity,
        ),
    ] {
        assert!(
            main.declarations.iter().any(|value| value.target == target),
            "missing {target:?}"
        );
    }
    let exported = inventory
        .modules
        .iter()
        .find(|value| value.module == "brand.veac")
        .unwrap()
        .declarations
        .iter()
        .find(|value| value.target == SourceNodeRef::function("brand.veac", "value"))
        .unwrap();
    assert!(exported.source.starts_with("export fn"));
}

#[test]
fn named_impl_blocks_for_one_receiver_have_distinct_stable_targets() {
    let source = r#"struct Card {}
impl Card @first { fn first(self) -> time { 1s } }
impl Card @second { fn second(self) -> time { 2s } }
fn main(context: Context) -> Project { context.empty_project() }"#;
    let sources = BTreeMap::from([("main.veac".into(), source.into())]);
    let index = super::test_support::index(&sources);
    let revision = super::test_revision(&index);
    let inventory = index.inventory(&revision).unwrap();
    let declarations = &inventory.modules[0].declarations;
    for target in [
        SourceNodeRef::implementation("main.veac", "Card", "first"),
        SourceNodeRef::implementation("main.veac", "Card", "second"),
    ] {
        assert!(declarations.iter().any(|value| value.target == target));
    }
}

#[test]
fn duplicate_impl_identity_for_one_receiver_fails_closed() {
    let source = r#"struct Card {}
impl Card @shared { fn first(self) -> time { 1s } }
impl Card @shared { fn second(self) -> time { 2s } }"#;
    let error = SourceIndex::build_snapshot(&BTreeMap::from([("main.veac".into(), source.into())]))
        .unwrap_err();
    assert_eq!(error.as_slice()[0].code, "SOURCE_INDEX_AMBIGUOUS_TARGET");
}
