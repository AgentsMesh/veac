use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_source_edit_path, compile_path, SourceTransactionError};
use veac_lang::source_edit::{
    ExpressionSite, ExpressionSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

const MODULE: &str = "module { export const time imported = 1s; }\n";
const ENTRY: &str = r#"import "./timing.veac" as timing;
const time local = 1s;
project atomic-edits {
  settings {
    timebase 1/1000; canvas 640px by 360px;
    frame-rate 30fps; sample-rate 48000hz;
  }
  entry sequence main;
  sequence main { layer visual content { item sample {
    source generated transparent; record { at 0s; duration ${local + timing.imported}; }
    modifiers {
      effect key { type video.luma_key; parameter threshold 0.5; }
    }
  } } }
}"#;

#[test]
fn cross_module_batch_is_rejected_without_changing_either_file() {
    let fixture = Fixture::new();
    let mut batch = fixture.batch("op_cross_module");
    batch.operations = vec![
        operation(
            SourceNodeRef::constant("main.veac", "local"),
            ExpressionSite::ConstantValue,
            "2s",
        ),
        operation(
            SourceNodeRef::constant("timing.veac", "imported"),
            ExpressionSite::ConstantValue,
            "3s",
        ),
    ];
    assert!(matches!(
        apply_source_edit_path(&fixture.entry, &batch),
        Err(SourceTransactionError::MultipleModules)
    ));
    fixture.assert_unchanged();
}

#[test]
fn compatible_but_missing_expression_site_reports_target_not_found() {
    let fixture = Fixture::new();
    let mut batch = fixture.batch("op_missing_site");
    batch.operations.push(operation(
        SourceNodeRef::item("main.veac", "atomic-edits", "main", "content", "sample"),
        ExpressionSite::ItemEnabled,
        "enabled",
    ));
    assert!(matches!(
        apply_source_edit_path(&fixture.entry, &batch),
        Err(SourceTransactionError::TargetNotFound { operation: 0 })
    ));
    fixture.assert_unchanged();
}

#[test]
fn canonical_ir_failure_is_a_lowering_error_and_keeps_source_bytes() {
    let fixture = Fixture::new();
    let mut batch = fixture.batch("op_invalid_canonical_value");
    batch.operations.push(operation(
        SourceNodeRef::modifier(
            "main.veac",
            "atomic-edits",
            "main",
            "content",
            "sample",
            "key",
        ),
        ExpressionSite::ModifierParameter {
            parameter: "threshold".into(),
        },
        "1.5",
    ));
    let error = apply_source_edit_path(&fixture.entry, &batch).unwrap_err();
    let SourceTransactionError::Lowering(diagnostics) = error else {
        panic!("unexpected source transaction error: {error}");
    };
    assert!(diagnostics
        .as_slice()
        .iter()
        .any(|value| value.code == "AUTHORING_LOWER_IR_VALIDATION"));
    fixture.assert_unchanged();
}

#[test]
fn successful_module_preview_preserves_every_untouched_module_byte() {
    let fixture = Fixture::new();
    let mut batch = fixture.batch("op_imported_only");
    batch.operations.push(operation(
        SourceNodeRef::constant("timing.veac", "imported"),
        ExpressionSite::ConstantValue,
        "2500ms",
    ));
    let preview = apply_source_edit_path(&fixture.entry, &batch).unwrap();
    assert_eq!(preview.module, "timing.veac");
    assert_eq!(preview.previous_modules(), ["main.veac", "timing.veac"]);
    assert_eq!(preview.compiled.sources()["main.veac"], ENTRY);
    assert_eq!(preview.previous_source(), MODULE);
    assert_eq!(preview.source(), MODULE.replace("1s", "2500ms"));
    fixture.assert_unchanged();
}

struct Fixture {
    _temp: tempfile::TempDir,
    entry: std::path::PathBuf,
    module: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let entry = temp.path().join("main.veac");
        let module = temp.path().join("timing.veac");
        fs::write(&entry, ENTRY).unwrap();
        fs::write(&module, MODULE).unwrap();
        Self {
            _temp: temp,
            entry,
            module,
        }
    }

    fn batch(&self, id: &str) -> SourceEditBatch {
        let revision = compile_path(&self.entry)
            .unwrap()
            .source_index()
            .unwrap()
            .revision()
            .clone();
        SourceEditBatch::new(veac_ir::OperationId::new(id).unwrap(), revision)
    }

    fn assert_unchanged(&self) {
        assert_eq!(fs::read_to_string(&self.entry).unwrap(), ENTRY);
        assert_eq!(fs::read_to_string(&self.module).unwrap(), MODULE);
    }
}

fn operation(target: SourceNodeRef, site: ExpressionSite, source: &str) -> SourceEditOperation {
    SourceEditOperation::SetExpression {
        target,
        site,
        expression: ExpressionSource {
            source: source.into(),
        },
    }
}
