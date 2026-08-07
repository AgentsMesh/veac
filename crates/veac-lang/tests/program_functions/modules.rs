use std::fs;

use tempfile::tempdir;
use veac_lang::program::build_path;

use super::support::{project_with, result_duration, validate_canonical};

const TIMING: &str = r#"module {
  fn helper(value: time) -> time { value * 2.0 }
  export fn padded(value: time) -> time { helper(value) + 250ms }
}"#;

#[test]
fn exported_qualified_function_retains_its_private_helper() {
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("timing.veac"), TIMING).unwrap();
    let source = project_with("import \"./timing.veac\" as timing;", "timing.padded(1s)");
    let entry = temp.path().join("main.veac");
    fs::write(&entry, source).unwrap();

    let built = build_path(&entry).unwrap();
    assert_eq!(result_duration(&built), "2250ms");
    validate_canonical(&built);
}

#[test]
fn private_module_function_is_not_visible_to_the_importer() {
    let temp = tempdir().unwrap();
    fs::write(
        temp.path().join("timing.veac"),
        "module { fn helper(value: time) -> time { value } }",
    )
    .unwrap();
    let source = project_with("import \"./timing.veac\" as timing;", "timing.helper(1s)");
    let entry = temp.path().join("main.veac");
    fs::write(&entry, source).unwrap();

    let diagnostics = build_path(&entry).unwrap_err();
    assert_eq!(diagnostics.as_slice()[0].path, "main.veac");
    let message = &diagnostics.as_slice()[0].message;
    assert!(
        message.contains("unknown function `timing.helper`"),
        "{message}"
    );
}

#[test]
fn runtime_error_points_to_the_innermost_private_helper_definition() {
    const FAILING: &str = r#"module {
  fn divide(value: scalar) -> scalar { 1.0 / value }
  fn helper(value: scalar) -> scalar { divide(value) }
  export fn inverse(value: scalar) -> scalar { helper(value) }
}"#;
    let temp = tempdir().unwrap();
    fs::write(temp.path().join("math.veac"), FAILING).unwrap();
    let source = project_with("import \"./math.veac\" as math;", "1s * math.inverse(0.0)");
    let entry = temp.path().join("main.veac");
    fs::write(&entry, source).unwrap();

    let diagnostics = build_path(&entry).unwrap_err();
    let error = &diagnostics.as_slice()[0];
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_RUNTIME");
    assert_eq!(error.path, "math.veac");
    assert_eq!(&FAILING[error.span.start..error.span.end], "1.0 / value");
    assert!(error.message.contains("in function `divide`"));
    assert!(error.message.contains("called from `helper`"));
    assert!(error.message.contains("called from `inverse`"));
    assert!(error.message.contains("called from main.veac"));
}
