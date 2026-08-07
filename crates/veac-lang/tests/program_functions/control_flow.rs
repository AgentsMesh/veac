use veac_lang::program::build_source;

use super::support::{project_with, result_duration, validate_canonical};

fn evaluated_duration(declarations: &str, expression: &str) -> String {
    let source = project_with(declarations, expression);
    let built = build_source(&source).unwrap();
    validate_canonical(&built);
    result_duration(&built)
}

#[test]
fn immutable_let_and_nested_shadow_drive_the_tail_result() {
    let declarations = r#"fn choose(value: time, enabled: bool) -> time {
  let padded = value + 250ms;
  {
    let padded = if enabled && padded >= 2s { padded } else { 2s };
    padded + 500ms
  }
}"#;

    assert_eq!(
        evaluated_duration(declarations, "choose(2s, true)"),
        "2750ms"
    );
}

#[test]
fn ordering_equality_and_boolean_precedence_select_typed_branches() {
    let declarations = r#"fn classify(value: time, expected: time) -> time {
  let in_range = value >= 2s && value <= 4s;
  let same = value == expected;
  if in_range && !same || value != expected { value } else { 1s }
}"#;

    assert_eq!(evaluated_duration(declarations, "classify(3s, 2s)"), "3s");
    assert_eq!(evaluated_duration(declarations, "classify(3s, 3s)"), "1s");
}

#[test]
fn logical_and_conditional_paths_execute_only_when_selected() {
    let declarations = r#"fn explode() -> bool { 1.0 / 0.0 == 0.0 }
fn guarded(enabled: bool) -> time {
  let skipped_and = false && explode();
  let skipped_or = true || explode();
  if enabled && !skipped_and && skipped_or {
    2s
  } else {
    if explode() { 3s } else { 4s }
  }
}"#;

    assert_eq!(evaluated_duration(declarations, "guarded(true)"), "2s");
}
