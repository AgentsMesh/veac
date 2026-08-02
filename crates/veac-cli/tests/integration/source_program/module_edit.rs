use super::*;

#[test]
fn source_edit_updates_the_target_module_and_protects_the_source_graph() {
    let temp = tempdir().unwrap();
    let entry_source = format!(
        "import \"./brand.veac\" as brand;\n{}",
        GENERATED_SOURCE.replace("duration 200ms", "duration ${brand.duration}")
    );
    let entry = source_file(&temp, &entry_source);
    let module = temp.path().join("brand.veac");
    std::fs::write(&module, "module { export const time duration = 200ms; }\n").unwrap();
    let mut edit = batch(revision(&entry), "750ms");
    let SourceEditOperation::SetExpression { target, .. } = &mut edit.operations[0];
    target.module = "brand.veac".into();
    let batch_path = temp.path().join("module-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&edit).unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            entry.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"module\": \"brand.veac\""));
    assert_eq!(std::fs::read_to_string(&entry).unwrap(), entry_source);
    assert!(std::fs::read_to_string(&module).unwrap().contains("750ms"));

    let mut next = batch(revision(&entry), "1s");
    let SourceEditOperation::SetExpression { target, .. } = &mut next.operations[0];
    target.module = "brand.veac".into();
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&next).unwrap(),
    )
    .unwrap();
    veac()
        .args([
            "source-edit",
            entry.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--output",
            entry.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
    assert_eq!(std::fs::read_to_string(entry).unwrap(), entry_source);
}

#[test]
fn explicit_target_output_uses_the_in_place_transaction() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let batch_path = temp.path().join("source-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch(
            revision(&source),
            "900ms",
        ))
        .unwrap(),
    )
    .unwrap();

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--output",
            source.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(std::fs::read_to_string(source).unwrap().contains("900ms"));
    assert!(temp.path().join(".veac-source.lock").is_file());
}
