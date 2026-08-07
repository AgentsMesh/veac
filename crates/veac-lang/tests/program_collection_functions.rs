use std::fs;

use tempfile::tempdir;
use veac_lang::program::{build_path, build_source};

#[path = "program_functions/support.rs"]
mod support;

#[test]
fn closed_collection_operations_execute_through_domain_lowering() {
    let declarations = r#"fn aggregate(values: range<int>) -> time {
  let mapped = map(values, fn(value: int) -> time effect pure { 100ms });
  let selected = filter(mapped, fn(value: time) -> bool effect pure { value >= 100ms });
  let shifted = for value in selected { value + 10ms };
  fold(shifted, 0ms, fn(total: time, value: time) -> time effect pure { total + value })
}
fn count(labels: map<text, int>) -> int {
  fold(map(labels, fn(entry: (text, int)) -> int effect pure { 1 }), 0,
    fn(total: int, value: int) -> int effect pure { total + value })
}"#;
    let expression = r#"if count(#{"b": 2, "a": 1}) == 2 {
  aggregate(0 .. 3)
} else { 1s }"#;
    let built = build_source(&support::project_with(declarations, expression)).unwrap();
    assert_eq!(support::result_duration(&built), "330ms");
    support::validate_canonical(&built);
}

#[test]
fn imported_callback_body_retains_a_later_private_helper() {
    let module = r#"module {
  export fn aggregate(values: range<int>) -> time {
    fold(map(values, fn(value: int) -> time effect pure { private(value) }), 0ms,
      fn(total: time, value: time) -> time effect pure { total + value })
  }
  fn private(value: int) -> time { 100ms }
}"#;
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("collection.veac"), module).unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(
        &entry,
        support::project_with(
            "import \"./collection.veac\" as collection;",
            "collection.aggregate(0 .. 3)",
        ),
    )
    .unwrap();

    let built = build_path(&entry).unwrap();
    assert_eq!(support::result_duration(&built), "300ms");
    support::validate_canonical(&built);
}

#[test]
fn callback_signatures_type_empty_inputs_and_fold_accumulators() {
    let declarations = r#"fn empty_values() -> list<int> {
  fold([], [], fn(acc: list<int>, value: int) -> list<int> effect pure { acc })
}"#;
    let expression =
        "if map([], fn(value: int) -> int effect pure { value }) == empty_values() { 250ms } else { 1s }";
    let built = build_source(&support::project_with(declarations, expression)).unwrap();
    assert_eq!(support::result_duration(&built), "250ms");
    support::validate_canonical(&built);
}
