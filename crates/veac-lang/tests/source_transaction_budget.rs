use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path, SourceTransactionError};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditError, SourceEditOperation, SourceNodeRef,
    MAX_SOURCE_EDIT_OUTPUT_BYTES,
};

#[path = "program_functions/support.rs"]
mod support;

#[test]
fn transaction_rejects_oversized_edited_module_before_overlay_compilation() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let source = source_at_module_limit();
    fs::write(&entry, &source).unwrap();
    let built = build_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_source_output_budget").unwrap(),
        built.source_revision().unwrap(),
    );
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "duration"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 1000ms }".to_owned(),
        },
    });

    assert!(matches!(
        apply_executable_source_edit_path(&entry, &batch),
        Err(SourceTransactionError::Contract(
            SourceEditError::EditedSourceTooLarge { .. }
        ))
    ));
    assert_eq!(fs::read(&entry).unwrap(), source.as_bytes());
}

fn source_at_module_limit() -> String {
    let mut source = support::project_with("fn duration() -> time { 1s }", "duration()");
    source.push_str("\n//");
    source.push_str(&"x".repeat(MAX_SOURCE_EDIT_OUTPUT_BYTES - source.len()));
    assert_eq!(source.len(), MAX_SOURCE_EDIT_OUTPUT_BYTES);
    source
}
