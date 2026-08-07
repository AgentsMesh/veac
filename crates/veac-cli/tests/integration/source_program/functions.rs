use super::*;

const CONTROL_FUNCTION: &str = r#"fn stretch(value: time, enabled: bool) -> time {
  let doubled = value * 2.0;
  if enabled && doubled >= 200ms { doubled } else { 200ms }
}"#;

fn source_with_function(function: &str, duration: &str) -> String {
    format!(
        "{function}\n{}",
        GENERATED_SOURCE.replace("during(0s, 200ms)", &format!("during(0s, {duration})"))
    )
}

fn function_source() -> String {
    source_with_function(CONTROL_FUNCTION, "stretch(150ms, true)")
}

#[test]
fn check_executes_functions_without_rendering_media() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &function_source());
    veac()
        .args(["check", source.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Executable source is valid"));
}

#[test]
fn build_executes_functions_and_emits_valid_canonical_ir() {
    let temp = tempdir().unwrap();
    let output = compile_ir(&temp, &function_source());
    let json = std::fs::read_to_string(output).unwrap();
    let envelope = veac_ir::decode_canonical_json(&json).unwrap();
    veac_ir::validate(&envelope).unwrap();
    assert!(envelope.project.id.as_str().starts_with("prj_"));
    let duration = envelope.project.sequences[0].tracks[0].clips[0]
        .record_range
        .duration;
    let expected = veac_ir::RationalTime::new(180, 600).unwrap();
    assert_eq!(
        duration.partial_cmp(&expected),
        Some(std::cmp::Ordering::Equal)
    );
}

#[test]
fn check_reports_an_invalid_unselected_branch_at_its_authored_line() {
    let temp = tempdir().unwrap();
    let invalid = r#"fn broken(enabled: bool) -> time {
  if enabled {
    200ms
  } else {
    missing
  }
}"#;
    let source = source_file(&temp, &source_with_function(invalid, "broken(true)"));
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
    assert_eq!(
        envelope["diagnostics"][0]["code"],
        "PROGRAM_FUNCTION_EXPRESSION"
    );
    assert_eq!(envelope["diagnostics"][0]["source_span"]["line"], 5);
    assert!(envelope["diagnostics"][0]["message"]
        .as_str()
        .unwrap()
        .contains("unknown symbol `missing`"));
}
