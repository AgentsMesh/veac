use super::*;

fn source_with(declarations: &str, duration: &str) -> String {
    format!(
        "{declarations}\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", &format!("during(0s, {duration})"))
    )
}

#[test]
fn check_accepts_ranges_through_function_boundaries() {
    let declarations = r#"fn choose(value: range<int>) -> time {
  if value == (0 .. 6 by 2) { 300ms } else { 200ms }
}"#;
    let source = source_with(declarations, "choose(0 .. 6 by 2)");
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source);

    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));
}

#[test]
fn check_reports_zero_range_step_at_the_authored_line() {
    let declarations = "fn broken() -> time {\n  let value = 0 .. 6 by 0;\n  200ms\n}";
    let source = source_with(declarations, "broken()");
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source);
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
    let envelope: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    let diagnostic = &envelope["diagnostics"][0];
    assert_eq!(diagnostic["code"], "PROGRAM_EXECUTABLE_RUNTIME");
    assert_eq!(diagnostic["source_span"]["line"], 2);
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("EXPRESSION_RANGE_STEP"));
}
