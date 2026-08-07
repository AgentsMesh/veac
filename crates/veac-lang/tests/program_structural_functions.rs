use std::fs;

use tempfile::tempdir;
use veac_lang::program::{build_path, build_source};

#[path = "program_functions/support.rs"]
mod support;

const STRUCTURAL_FUNCTIONS: &str = r#"fn keep_list(value: list<int>) -> list<int> { value }
fn keep_pair(value: (int, text)) -> (int, text) { value }
fn keep_map(value: map<text, int>) -> map<text, int> { value }
fn keep_ids(value: map<identifier, int>) -> map<identifier, int> { value }
fn structural_duration() -> time {
  let empty_list: list<int> = [];
  let empty_map: map<text, int> = #{};
  let list = keep_list([1, 2, 3]);
  let pair = keep_pair((1, "x"));
  let labels = keep_map(#{"b": 2, "a": 1});
  let ids = keep_ids(#{identifier("b"): 2, identifier("a"): 1});
  if list == [1, 2, 3]
    && pair == (1, "x")
    && labels == #{"a": 1, "b": 2}
    && ids == #{identifier("a"): 1, identifier("b"): 2}
    && empty_list == keep_list([])
    && empty_map == keep_map(#{} ) {
    3s
  } else {
    1s
  }
}"#;

#[test]
fn structural_parameters_returns_and_expected_empty_types_execute() {
    let source = support::project_with(STRUCTURAL_FUNCTIONS, "structural_duration()");
    let built = build_source(&source).unwrap();
    assert_eq!(support::result_duration(&built), "3s");
    support::validate_canonical(&built);
}

#[test]
fn imported_structural_function_retains_a_later_private_helper() {
    let module = r#"module {
  export fn normalize(values: list<int>) -> list<int> { private(values) }
  fn private(values: list<int>) -> list<int> { values }
}"#;
    let declarations = r#"import "./lists.veac" as lists;
fn imported_duration(values: list<int>) -> time {
  if lists.normalize(values) == [1, 2, 3] { 3s } else { 1s }
}"#;
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("lists.veac"), module).unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(
        &entry,
        support::project_with(declarations, "imported_duration([1,2,3])"),
    )
    .unwrap();

    let built = build_path(&entry).unwrap();
    assert_eq!(support::result_duration(&built), "3s");
    support::validate_canonical(&built);
}
