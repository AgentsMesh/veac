use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_source_edit_path, compile_path, SourceTransactionError};
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditError, SourceEditOperation,
    SourceNodeRef, MAX_SOURCE_EDIT_OUTPUT_BYTES,
};

#[test]
fn transaction_rejects_oversized_edited_module_before_overlay_compilation() {
    let temp = tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    let source = source_at_module_limit();
    fs::write(&entry, &source).unwrap();
    let compiled = compile_path(&entry).unwrap();
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_source_output_budget").unwrap(),
        compiled.source_index().unwrap().revision().clone(),
    );
    batch.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::constant("main.veac", "title"),
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: "\"longer\"".to_owned(),
        },
    });

    assert!(matches!(
        apply_source_edit_path(&entry, &batch),
        Err(SourceTransactionError::Contract(
            SourceEditError::EditedSourceTooLarge { .. }
        ))
    ));
    assert_eq!(fs::read(&entry).unwrap(), source.as_bytes());
}

fn source_at_module_limit() -> String {
    let mut source = r#"const text title = "x";
project source-edit-budget {
  settings {
    timebase 1/1000; canvas 64px by 64px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main { layer visual content { item frame {
    source generated transparent; record { at 0s; duration 1s; }
  } } }
}
//"#
    .to_owned();
    source.push_str(&"x".repeat(MAX_SOURCE_EDIT_OUTPUT_BYTES - source.len()));
    assert_eq!(source.len(), MAX_SOURCE_EDIT_OUTPUT_BYTES);
    source
}
