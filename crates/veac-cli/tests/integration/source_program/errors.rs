use super::*;

#[cfg(unix)]
#[test]
fn source_revision_rejects_a_directory_as_a_source_file() {
    let temp = tempdir().unwrap();
    veac()
        .args(["source-revision", temp.path().to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("INVALID_SOURCE_PATH"))
        .stderr(predicate::str::contains("not a regular file"));
}

#[test]
fn formatter_canonicalizes_a_standalone_module() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, "module { export const time duration = 200ms; }\n");
    let original = std::fs::read_to_string(&source).unwrap();

    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FORMAT_REQUIRED"));
    veac()
        .args(["fmt", source.to_str().unwrap()])
        .assert()
        .success();
    assert_eq!(
        std::fs::read_to_string(&source).unwrap(),
        "module {\n  export const time duration = 200ms;\n}\n"
    );
    assert_ne!(std::fs::read_to_string(&source).unwrap(), original);
    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
}

#[test]
fn formatter_preserves_program_import_diagnostics() {
    let temp = tempdir().unwrap();
    let source = source_file(
        &temp,
        &format!("import \"./missing.veac\" as missing;\n{GENERATED_SOURCE}"),
    );

    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("PROGRAM_IMPORT_LOAD"));
}

#[test]
fn source_edit_rejects_non_json_batches_before_editing() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let original = std::fs::read_to_string(&source).unwrap();
    let batch_path = temp.path().join("source-edit.json");
    std::fs::write(&batch_path, "not json").unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SOURCE_EDIT_BATCH_JSON"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
}
