use std::fs;

use tempfile::tempdir;
use veac_lang::program::{build_path, build_source, prepare_source};

#[path = "program_functions/support.rs"]
mod support;

#[test]
fn named_functions_accept_and_return_exact_function_types() {
    let declarations = r#"fn invoke(callback: fn(time) -> time effect pure, value: time) -> time {
  callback(value)
}
fn make(offset: time) -> fn(time) -> time effect pure {
  fn(value: time) -> time effect pure { value + offset }
}"#;
    let built = build_source(&support::project_with(declarations, "invoke(make(1s), 2s)")).unwrap();
    assert_eq!(support::result_duration(&built), "3s");
    support::validate_canonical(&built);
}

#[test]
fn imported_closure_retains_its_capture_and_later_private_helper() {
    let module = r#"module {
  export fn make(offset: time) -> fn(time) -> time effect pure {
    fn(value: time) -> time effect pure { private(value, offset) }
  }
  fn private(value: time, offset: time) -> time { value + offset }
}"#;
    let declarations = r#"import "./timing.veac" as timing;
fn imported_duration(value: time) -> time { timing.make(2s)(value) }"#;
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("timing.veac"), module).unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(
        &entry,
        support::project_with(declarations, "imported_duration(1s)"),
    )
    .unwrap();

    let built = build_path(&entry).unwrap();
    assert_eq!(support::result_duration(&built), "3s");
    support::validate_canonical(&built);
}

#[test]
fn function_values_cannot_reach_a_domain_time_leaf() {
    let callback = "fn(value: time) -> time effect pure { value }";
    let source = support::project_with("", callback);
    let diagnostics = prepare_source(&source).unwrap_err();
    assert_eq!(
        diagnostics.as_slice()[0].code,
        "PROGRAM_FUNCTION_EXPRESSION"
    );
    let diagnostic = &diagnostics.as_slice()[0];
    assert_eq!(
        &source[diagnostic.span.start..diagnostic.span.end],
        callback
    );
    assert!(
        diagnostic.message.contains("fn(time) -> time effect pure"),
        "{diagnostic:?}"
    );
}
