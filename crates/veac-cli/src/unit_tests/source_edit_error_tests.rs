use tempfile::{tempdir, TempDir};
use veac_ir::OperationId;
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
    SourceRevision,
};

use super::support::{source_file, EXECUTABLE_SOURCE};

#[test]
fn source_edit_maps_malformed_batches() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, EXECUTABLE_SOURCE);
    let batch = temp.path().join("malformed-source-edit.json");
    std::fs::write(&batch, r#"{"atomic":true}"#).unwrap();

    let error = crate::commands::source_edit(&source, &batch, None, &[], None, true).unwrap_err();
    assert!(error.to_string().contains("SOURCE_EDIT_BATCH_JSON"));
}

#[test]
fn source_edit_maps_program_failures_from_the_transaction() {
    let temp = tempdir().unwrap();
    let batch = write_valid_batch(&temp);
    let missing = temp.path().join("missing.veac");

    let error = crate::commands::source_edit(&missing, &batch, None, &[], None, true).unwrap_err();
    assert!(error.to_string().contains("PATH_UNAVAILABLE"));
}

fn write_valid_batch(temp: &TempDir) -> std::path::PathBuf {
    let mut batch = SourceEditBatch::new(
        OperationId::new("op_missing_source").unwrap(),
        SourceRevision {
            source_graph_sha256: "0".repeat(64),
        },
    );
    batch.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::constant("main.veac", "duration"),
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: "1s".into(),
        },
    });
    let path = temp.path().join("source-edit.json");
    let json = veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap();
    std::fs::write(&path, json).unwrap();
    path
}
