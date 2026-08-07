use super::*;

#[test]
fn formatter_recognizes_program_syntax_without_line_prefix_heuristics() {
    let temp = tempdir().unwrap();
    let source = source_file(
        &temp,
        &program_source().replacen("const time", "/* typed declaration */ const\ntime", 1),
    );
    let original = std::fs::read_to_string(&source).unwrap();

    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FORMAT_REQUIRED"));
    veac()
        .args(["fmt", source.to_str().unwrap(), "--stdout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("/* typed declaration */"));
    assert_eq!(std::fs::read_to_string(&source).unwrap(), original);
    veac()
        .args(["fmt", source.to_str().unwrap()])
        .assert()
        .success();
    let formatted = std::fs::read_to_string(&source).unwrap();
    assert_ne!(formatted, original);
    assert!(formatted.contains("/* typed declaration */"));
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
}
