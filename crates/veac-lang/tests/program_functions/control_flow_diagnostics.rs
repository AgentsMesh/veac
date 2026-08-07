use veac_lang::program::{prepare_source, Diagnostic};

use super::support::project_with;

fn first_error(declarations: &str) -> (String, Diagnostic) {
    let source = project_with(declarations, "1s");
    let error = prepare_source(&source)
        .expect_err("control-flow fixture must fail")
        .as_slice()[0]
        .clone();
    (source, error)
}

#[test]
fn an_unselected_branch_is_still_checked_at_its_absolute_span() {
    let declarations = r#"fn broken(enabled: bool) -> time {
  if enabled { 1s } else { missing }
}"#;
    let (source, error) = first_error(declarations);

    assert_eq!(error.code, "PROGRAM_FUNCTION_EXPRESSION");
    assert_eq!(error.path, "main.veac");
    assert_eq!(&source[error.span.start..error.span.end], "missing");
    assert!(error.message.contains("unknown symbol `missing`"));
}

#[test]
fn a_short_circuited_operand_is_still_statically_checked() {
    let (source, error) = first_error("fn broken() -> bool { false && missing }");

    assert_eq!(error.code, "PROGRAM_FUNCTION_EXPRESSION");
    assert_eq!(&source[error.span.start..error.span.end], "missing");
}

#[test]
fn if_requires_a_bool_condition_and_one_branch_type() {
    for (declarations, expected) in [
        (
            "fn broken() -> time { if 1 { 1s } else { 2s } }",
            "if condition requires bool",
        ),
        (
            "fn broken(value: bool) -> time { if value { 1s } else { 2px } }",
            "if branches must have one type",
        ),
    ] {
        let (_, error) = first_error(declarations);
        assert_eq!(error.code, "PROGRAM_FUNCTION_EXPRESSION");
        assert!(error.message.contains(expected), "{}", error.message);
    }
}

#[test]
fn one_block_cannot_redeclare_a_local() {
    let (source, error) =
        first_error("fn broken() -> time { let value = 1s; let value = 2s; value }");

    assert_eq!(error.code, "PROGRAM_FUNCTION_EXPRESSION");
    assert_eq!(&source[error.span.start..error.span.end], "value");
    assert!(error.message.contains("declared more than once"));
}

#[test]
fn comparisons_and_logical_operators_reject_wrong_types() {
    for (declarations, expected) in [
        (
            "fn broken() -> bool { 1s < 2px }",
            "ordering requires one numeric kind",
        ),
        (
            "fn broken() -> bool { 1s == 2px }",
            "cannot compare time and length",
        ),
        (
            "fn broken() -> bool { true && 1.0 }",
            "logical operator requires bool",
        ),
    ] {
        let (_, error) = first_error(declarations);
        assert_eq!(error.code, "PROGRAM_FUNCTION_EXPRESSION");
        assert!(error.message.contains(expected), "{}", error.message);
    }
}
