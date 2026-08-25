use std::collections::BTreeMap;

use crate::source_edit::SourceNodeRef;

const SOURCE: &str = r#"const text label = "中文🙂";
fn amount() -> time {
  let offset = 1s;
  var current = offset;
  set current = current + 1s;
  current
}
struct Timing { duration: time, }
impl Timing @timing {
  fn amount(self) -> time {
    let offset = 2s;
    offset
  }
}"#;

#[test]
fn statement_inventory_keeps_exact_utf8_byte_ranges_and_semicolons() {
    let index = super::test_support::index(&sources());
    let revision = super::test_revision(&index);
    let node = index
        .inventory(&revision)
        .unwrap()
        .nodes
        .into_iter()
        .find(|node| node.target == function())
        .unwrap();
    let authored = node
        .statements
        .iter()
        .map(|statement| statement.source.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        authored,
        [
            "let offset = 1s;",
            "var current = offset;",
            "set current = current + 1s;"
        ]
    );
    for statement in node.statements {
        assert_eq!(
            &SOURCE[statement.range.start..statement.range.end],
            statement.source
        );
        assert!(statement.source.ends_with(';'));
    }
}

#[test]
fn identical_function_and_method_paths_are_owner_isolated_and_stable() {
    let first = super::test_support::index(&sources());
    let second = super::test_support::index(&sources());
    let first_revision = super::test_revision(&first);
    let second_revision = super::test_revision(&second);
    let function_site = first
        .inventory(&first_revision)
        .unwrap()
        .nodes
        .into_iter()
        .find(|node| node.target == function())
        .unwrap()
        .statements[0]
        .site
        .clone();
    let method_site = first
        .inventory(&first_revision)
        .unwrap()
        .nodes
        .into_iter()
        .find(|node| node.target == method())
        .unwrap()
        .statements[0]
        .site
        .clone();
    assert_eq!(function_site, method_site);
    assert_eq!(
        first.statement(&function(), &function_site).unwrap().source,
        "let offset = 1s;"
    );
    assert_eq!(
        first.statement(&method(), &method_site).unwrap().source,
        "let offset = 2s;"
    );
    assert_eq!(
        first.statement(&function(), &function_site),
        second.statement(&function(), &function_site)
    );
    assert_eq!(
        first.inventory(&first_revision).unwrap(),
        second.inventory(&second_revision).unwrap()
    );
}

fn function() -> SourceNodeRef {
    SourceNodeRef::function("main.veac", "amount")
}

fn method() -> SourceNodeRef {
    SourceNodeRef::method("main.veac", "Timing", "amount")
}

fn sources() -> BTreeMap<String, String> {
    BTreeMap::from([("main.veac".to_owned(), SOURCE.to_owned())])
}
