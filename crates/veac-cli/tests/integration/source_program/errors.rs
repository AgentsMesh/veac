use super::*;

#[cfg(unix)]
#[test]
fn source_revision_maps_an_unreadable_root_to_a_program_diagnostic() {
    veac()
        .args(["source-revision", "/"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("READ_FAILED"))
        .stderr(predicate::str::contains("not a regular file"));
}

#[test]
fn source_revision_reports_ambiguous_authored_targets() {
    let temp = tempdir().unwrap();
    let record = "        record { at 0s; duration 200ms; }";
    let duplicate_modifiers = concat!(
        "        record { at 0s; duration 200ms; }\n",
        "        modifiers {\n",
        "          effect duplicate { type video.blur; parameter radius 1; }\n",
        "          effect duplicate { type video.blur; parameter radius 2; }\n",
        "        }"
    );
    let source = source_file(
        &temp,
        &GENERATED_SOURCE.replacen(record, duplicate_modifiers, 1),
    );

    veac()
        .args(["source-revision", source.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SOURCE_INDEX_AMBIGUOUS_TARGET"));
}

#[test]
fn formatter_accepts_a_standalone_module_without_rewriting_it() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, "module { export const time duration = 200ms; }\n");
    let original = std::fs::read_to_string(&source).unwrap();

    veac()
        .args(["fmt", source.to_str().unwrap(), "--check"])
        .assert()
        .success();
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
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
fn source_edit_lowering_failure_never_changes_the_source() {
    let temp = tempdir().unwrap();
    let invalid = program_source().replace(
        "record { at 0s; duration ${duration}; }",
        concat!(
            "record { at 0s; duration ${duration}; }\n",
            "        mapping linear { from 0s; to ${duration}; }"
        ),
    );
    let source = source_file(&temp, &invalid);
    let original = std::fs::read_to_string(&source).unwrap();
    let batch_path = temp.path().join("source-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch(
            revision(&source),
            "400ms",
        ))
        .unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("AUTHORING_LOWER_MAPPING_SOURCE"));
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
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
