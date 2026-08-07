use veac_lang::program::build_source;

use super::support::project_with;

#[test]
fn call_arguments_execute_left_to_right_and_keep_definition_spans() {
    let declarations = r#"fn fail_left(value: scalar) -> time { 1s / value }
fn fail_right(value: scalar) -> time { 2s / value }
fn combine(left: time, right: time) -> time { left + right }
"#;
    let source = project_with(declarations, "combine(fail_left(0.0), fail_right(0.0))");
    let diagnostics = build_source(&source).unwrap_err();
    let error = &diagnostics.as_slice()[0];

    assert_eq!(error.code, "PROGRAM_EXECUTABLE_RUNTIME");
    assert_eq!(error.path, "main.veac");
    assert_eq!(&source[error.span.start..error.span.end], "1s / value");
    assert!(error.message.contains("in function `fail_left`"));
    assert!(!error.message.contains("in function `fail_right`"));
}
