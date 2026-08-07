use super::*;

fn source_with(declarations: &str, duration: &str) -> String {
    format!(
        "{declarations}\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", &format!("during(0s, {duration})"))
    )
}

#[test]
fn check_executes_nominal_construction_match_projection_and_methods() {
    let declarations = r#"struct Timing { duration: time, }
enum Choice { Exact { timing: Timing, }, Fallback, }
impl Timing @timing { fn padded(self) -> time { self.duration + 250ms } }
fn choose(value: Choice) -> time {
  match value {
    Choice.Exact { timing } => timing.padded(),
    Choice.Fallback => 1s,
  }
}
const Choice choice = Choice.Exact {
  timing: Timing { duration: 750ms, },
};"#;
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source_with(declarations, "choose(choice)"));
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));
}

#[test]
fn json_diagnostic_points_to_the_authored_non_exhaustive_match() {
    let declarations = r#"enum Choice { First, Second, }
fn choose(value: Choice) -> time {
  match value {
    Choice.First => 1s,
  }
}
const Choice choice = Choice.Second;"#;
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source_with(declarations, "choose(choice)"));
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
    assert_eq!(diagnostic["source_span"]["line"], 3);
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("EXPRESSION_NON_EXHAUSTIVE_MATCH"));
}

#[test]
fn json_diagnostic_points_to_the_authored_method_body() {
    let declarations = r#"struct Timing { duration: time, }
impl Timing @timing {
  fn broken(self) -> time { self.missing }
}"#;
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &source_with(declarations, "200ms"));
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
    assert_eq!(diagnostic["source_span"]["line"], 3);
    assert!(diagnostic["source_span"]["column"].as_u64().unwrap() > 1);
    assert!(diagnostic["message"]
        .as_str()
        .unwrap()
        .contains("EXPRESSION_UNKNOWN_FIELD"));
}
