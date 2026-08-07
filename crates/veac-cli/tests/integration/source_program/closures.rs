use super::*;

fn source_with(declarations: &str, duration: &str) -> String {
    format!(
        "{declarations}\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", &format!("during(0s, {duration})"))
    )
}

fn diagnostic(source: &str) -> serde_json::Value {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, source);
    let output = veac()
        .args([
            "--diagnostic-format",
            "json",
            "check",
            source.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    serde_json::from_slice(&output.stderr).unwrap()
}

#[test]
fn check_executes_closures_through_named_function_boundaries() {
    let declarations = r#"fn apply_callback(callback: fn(time) -> time effect pure, value: time) -> time {
  callback(value)
}
fn make(offset: time) -> fn(time) -> time effect pure {
  fn(value: time) -> time effect pure { value + offset }
}"#;
    let source = source_with(declarations, "apply_callback(make(100ms), 200ms)");
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source);
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));
}

#[test]
fn check_reports_closure_return_types_at_the_authored_line() {
    let declarations = r#"fn broken() -> fn(time) -> time effect pure {
  fn(value: time) -> time effect pure { 1 }
}"#;
    let envelope = diagnostic(&source_with(declarations, "200ms"));
    let diagnostic = &envelope["diagnostics"][0];
    assert_eq!(diagnostic["code"], "PROGRAM_FUNCTION_EXPRESSION");
    assert_eq!(diagnostic["source_span"]["line"], 2);
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("EXPRESSION_CLOSURE_RETURN_TYPE"));
}

#[test]
fn runtime_closure_failures_remain_top_level_program_expressions() {
    let declarations = r#"fn deferred() -> fn() -> int effect pure {
  fn() -> int effect pure { 9223372036854775807 + 1 }
}"#;
    let duration = "if deferred()() > 0 { 200ms } else { 300ms }";
    let envelope = diagnostic(&source_with(declarations, duration));
    let diagnostic = &envelope["diagnostics"][0];
    assert_eq!(diagnostic["code"], "PROGRAM_EXECUTABLE_RUNTIME");
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("EXPRESSION_OVERFLOW"));
}
