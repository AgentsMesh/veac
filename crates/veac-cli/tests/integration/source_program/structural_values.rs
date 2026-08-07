use super::*;

fn source_with(declarations: &str, duration: &str) -> String {
    format!(
        "{declarations}\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", &format!("during(0s, {duration})"))
    )
}

#[test]
fn check_accepts_structural_values_through_function_boundaries() {
    let declarations = r#"fn choose(
  values: list<int>,
  pair: (int, text),
  labels: map<text, int>
) -> time {
  let empty: list<int> = [];
  if values == [1, 2, 3]
    && pair == (1, "x")
    && labels == #{"a": 1}
    && empty == [] {
    300ms
  } else {
    200ms
  }
}"#;
    let source = source_with(declarations, "choose([1,2,3],(1,\"x\"),#{\"a\":1})");
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source);

    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));
}

#[test]
fn check_reports_heterogeneous_list_at_the_authored_line() {
    let declarations = "fn broken() -> list<int> {\n  [1, \"x\"]\n}";
    let source = source_with(declarations, "200ms");
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
    assert_eq!(diagnostic["code"], "PROGRAM_FUNCTION_EXPRESSION");
    assert_eq!(diagnostic["source_span"]["line"], 2);
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("EXPRESSION_LIST_ELEMENT_TYPE"));
}
