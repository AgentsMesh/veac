use super::support::*;

fn input_source(temp: &TempDir) -> std::path::PathBuf {
    source_file(
        temp,
        &format!(
            "input parameter duration: time;\n{}",
            EXECUTABLE_SOURCE.replacen("200ms", "duration", 1)
        ),
    )
}

#[test]
fn inline_input_overrides_manifest_for_build_and_check() {
    let temp = tempdir().unwrap();
    let source = input_source(&temp);
    let manifest = temp.path().join("inputs.json");
    std::fs::write(
        &manifest,
        r#"{"schema":"https://veac.dev/schemas/build-inputs","schema_version":1,"inputs":[{"name":"duration","value":{"type":"time","value":"250ms"}}]}"#,
    )
    .unwrap();
    let project = temp.path().join("project.json");
    veac()
        .args([
            "build",
            source.to_str().unwrap(),
            "--inputs",
            manifest.to_str().unwrap(),
            "--input",
            "duration=400ms",
            "--emit-ir",
            project.to_str().unwrap(),
        ])
        .assert()
        .success();
    veac()
        .args([
            "check",
            source.to_str().unwrap(),
            "--inputs",
            manifest.to_str().unwrap(),
            "--input",
            "duration=400ms",
        ])
        .assert()
        .success();
    let envelope =
        veac_ir::decode_canonical_json(&std::fs::read_to_string(project).unwrap()).unwrap();
    assert_eq!(
        envelope.project.sequences[0].tracks[0].clips[0]
            .record_range
            .duration
            .value,
        400
    );
}

#[test]
fn repeated_inline_values_support_enum_and_text_with_equals() {
    let temp = tempdir().unwrap();
    let source = source_file(
        &temp,
        &format!(
            "enum Locale {{ en, zh-Hans, }}\ninput parameter locale: Locale;\ninput parameter title: text;\n{EXECUTABLE_SOURCE}"
        ),
    );
    veac()
        .args([
            "check",
            source.to_str().unwrap(),
            "--input",
            "title=第一段=第二段",
            "--input",
            "locale=zh-Hans",
        ])
        .assert()
        .success();
}

#[test]
fn malformed_duplicate_unknown_missing_and_invalid_inline_values_fail_closed() {
    let temp = tempdir().unwrap();
    let source = source_file(
        &temp,
        &format!(
            "input parameter count: int;\ninput parameter enabled: bool;\n{EXECUTABLE_SOURCE}"
        ),
    );
    let cases: &[(&[&str], &str)] = &[
        (&["--input", "count"], "PROGRAM_INPUT_INLINE_SYNTAX"),
        (
            &["--input", "count=1", "--input", "count=2"],
            "PROGRAM_INPUT_DUPLICATE",
        ),
        (&["--input", "other=1"], "PROGRAM_INPUT_UNKNOWN"),
        (&["--input", "count=1"], "PROGRAM_INPUT_MISSING"),
        (
            &["--input", "count=wrong", "--input", "enabled=true"],
            "PROGRAM_INPUT_VALUE",
        ),
    ];
    for (arguments, code) in cases {
        veac()
            .arg("check")
            .arg(&source)
            .args(*arguments)
            .assert()
            .failure()
            .stderr(predicate::str::contains(*code));
    }
}
