use std::fs;

use tempfile::tempdir;
use veac_lang::program::{build_path, build_source, Diagnostic};

use super::support::project_with;

#[test]
fn constant_runtime_error_keeps_the_function_definition_span() {
    let source = project_with(
        r#"fn divide(value: scalar) -> time { 1s / value }
const time duration = divide(0.0);"#,
        "1s",
    );
    let error = source_error(&source);
    assert_definition(
        &error,
        "PROGRAM_CONST_EXPRESSION",
        "main.veac",
        &source,
        "1s / value",
    );
    assert!(error.message.contains("in function `divide`"));
}

#[test]
fn imported_function_error_ignores_the_caller_source_offset() {
    let math = r#"module {
  export fn divide(value: scalar) -> time { 1s / value }
}"#;
    let entry = project_with("import \"./math.veac\" as math;", "math.divide(0.0)");
    let error = path_error(&entry, &[("math.veac", math)]);
    assert_definition(
        &error,
        "PROGRAM_EXECUTABLE_RUNTIME",
        "math.veac",
        math,
        "1s / value",
    );
    assert!(error.message.contains("called from main.veac"));
}

fn source_error(source: &str) -> Diagnostic {
    build_source(source)
        .expect_err("fixture must fail")
        .as_slice()[0]
        .clone()
}

fn path_error(entry: &str, modules: &[(&str, &str)]) -> Diagnostic {
    let temp = tempdir().unwrap();
    let path = temp.path().join("main.veac");
    fs::write(&path, entry).unwrap();
    for (name, source) in modules {
        fs::write(temp.path().join(name), source).unwrap();
    }
    build_path(&path).expect_err("fixture must fail").as_slice()[0].clone()
}

fn assert_definition(error: &Diagnostic, code: &str, path: &str, source: &str, expected: &str) {
    assert_eq!(error.code, code);
    assert_eq!(error.path, path);
    assert_eq!(&source[error.span.start..error.span.end], expected);
}
