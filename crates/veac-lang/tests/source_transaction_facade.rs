use std::fs;

use tempfile::tempdir;
use veac_lang::program::{
    apply_executable_source_edit_path_with_root,
    apply_executable_source_edit_path_with_root_and_inputs,
    prepare_executable_source_edit_path_with_root,
    prepare_executable_source_edit_path_with_root_and_database, BuildInputManifestV1,
    CompilerDatabase, SourceTransactionError,
};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

#[path = "program_functions/support.rs"]
mod support;

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, SourceEditBatch) {
    let directory = tempdir().unwrap();
    let entry = directory.path().join("main.veac");
    fs::write(
        &entry,
        support::project_with("fn duration() -> time { 1s }", "duration()"),
    )
    .unwrap();
    let prepared = veac_lang::program::prepare_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_source_transaction_facade").unwrap(),
        prepared.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "duration"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 2500ms }".to_owned(),
        },
    });
    (directory, entry, batch)
}

#[test]
fn root_and_prepare_facades_share_the_same_transaction_contract() {
    let (directory, entry, batch) = fixture();
    let expected_root = directory.path().canonicalize().unwrap();

    let (root, preview) = apply_executable_source_edit_path_with_root(&entry, &batch).unwrap();
    assert_eq!(root, expected_root);
    assert_eq!(support::result_duration(&preview.built), "2500ms");

    let (root, candidate) = prepare_executable_source_edit_path_with_root(&entry, &batch).unwrap();
    assert_eq!(root, expected_root);
    let preview = candidate.execute(&BuildInputManifestV1::empty()).unwrap();
    assert_eq!(support::result_duration(&preview.built), "2500ms");
}

#[test]
fn explicit_inputs_and_database_facades_preserve_root_and_query_reuse() {
    let (directory, entry, batch) = fixture();
    let expected_root = directory.path().canonicalize().unwrap();
    let database = CompilerDatabase::default();

    let (root, candidate) =
        prepare_executable_source_edit_path_with_root_and_database(&entry, &batch, &database)
            .unwrap();
    assert_eq!(root, expected_root);
    candidate.execute(&BuildInputManifestV1::empty()).unwrap();
    let before = database.statistics();

    let (root, preview) = apply_executable_source_edit_path_with_root_and_inputs(
        &entry,
        &batch,
        &BuildInputManifestV1::empty(),
    )
    .unwrap();
    assert_eq!(root, expected_root);
    assert_eq!(support::result_duration(&preview.built), "2500ms");
    assert!(before.syntax_insertions > 0);
}

#[test]
fn missing_entry_is_reported_as_a_program_load_diagnostic() {
    let (_directory, entry, batch) = fixture();
    let missing = entry.with_file_name("missing.veac");
    let error = prepare_executable_source_edit_path_with_root(&missing, &batch).unwrap_err();
    let SourceTransactionError::Program(diagnostics) = error else {
        panic!("missing entry must be a program diagnostic");
    };
    assert_eq!(diagnostics.as_slice()[0].code, "PROGRAM_ENTRY_LOAD");
    assert_eq!(
        diagnostics.as_slice()[0].path,
        missing.display().to_string()
    );
}
