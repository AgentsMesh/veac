use std::fs;

use tempfile::tempdir;
use veac_lang::program::{build_path, build_source};

#[path = "program_functions/support.rs"]
mod support;

#[test]
fn range_parameters_and_returns_execute_through_domain_lowering() {
    let declarations = r#"fn keep(value: range<int>) -> range<int> { value }
fn choose(value: range<int>) -> time {
  if keep(value) == (0 .. 6 by 2) { 3s } else { 1s }
}"#;
    let built = build_source(&support::project_with(declarations, "choose(0 .. 6 by 2)")).unwrap();
    assert_eq!(support::result_duration(&built), "3s");
    support::validate_canonical(&built);
}

#[test]
fn imported_range_function_retains_a_later_private_helper() {
    let module = r#"module {
  export fn normalize(value: range<int>) -> range<int> { private(value) }
  fn private(value: range<int>) -> range<int> { value }
}"#;
    let declarations = r#"import "./ranges.veac" as ranges;
fn imported_duration(value: range<int>) -> time {
  if ranges.normalize(value) == (10 .. 0 by -2) { 5s } else { 1s }
}"#;
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("ranges.veac"), module).unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(
        &entry,
        support::project_with(declarations, "imported_duration(10 .. 0 by -2)"),
    )
    .unwrap();

    let built = build_path(&entry).unwrap();
    assert_eq!(support::result_duration(&built), "5s");
    support::validate_canonical(&built);
}
