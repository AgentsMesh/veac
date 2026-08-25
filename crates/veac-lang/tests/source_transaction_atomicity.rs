use std::fs;

use tempfile::tempdir;
use veac_lang::program::{apply_executable_source_edit_path, build_path, SourceTransactionError};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

#[path = "program_functions/support.rs"]
mod support;

const MODULE: &str = "module { export fn imported() -> time { 1s } }\n";

fn entry_source() -> String {
    support::project_with(
        "import \"./timing.veac\" as timing;\nfn local() -> time { 1s }",
        "local() + timing.imported()",
    )
}

#[test]
fn cross_module_batch_builds_one_atomic_preview_without_writing_files() {
    let fixture = Fixture::new();
    let mut batch = fixture.batch("op_cross_module");
    batch.operations = vec![
        operation(SourceNodeRef::function("main.veac", "local"), "{ 2s }"),
        operation(SourceNodeRef::function("timing.veac", "imported"), "{ 3s }"),
    ];
    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert_eq!(preview.changed_modules(), ["main.veac", "timing.veac"]);
    assert_eq!(preview.previous_source(), None);
    assert_eq!(preview.source(), None);
    assert_eq!(
        preview.changes()[0].source(),
        entry_source().replace("{ 1s }", "{ 2s }")
    );
    assert_eq!(
        preview.changes()[1].source(),
        MODULE.replace("{ 1s }", "{ 3s }")
    );
    assert_eq!(support::result_duration(&preview.built), "5s");
    fixture.assert_unchanged();
}

#[test]
fn missing_body_site_reports_target_not_found() {
    let fixture = Fixture::new();
    let mut batch = fixture.batch("op_missing_site");
    batch.operations.push(operation(
        SourceNodeRef::function("main.veac", "missing"),
        "{ 2s }",
    ));
    assert!(matches!(
        apply_executable_source_edit_path(&fixture.entry, &batch),
        Err(SourceTransactionError::TargetNotFound { operation: 0 })
    ));
    fixture.assert_unchanged();
}

#[test]
fn invalid_overlay_keeps_every_source_byte() {
    let fixture = Fixture::new();
    let mut batch = fixture.batch("op_invalid_overlay");
    batch.operations.push(operation(
        SourceNodeRef::function("timing.veac", "imported"),
        "{ missing }",
    ));
    assert!(matches!(
        apply_executable_source_edit_path(&fixture.entry, &batch),
        Err(SourceTransactionError::Program(_))
    ));
    fixture.assert_unchanged();
}

#[test]
fn successful_module_preview_preserves_every_untouched_module_byte() {
    let fixture = Fixture::new();
    let mut batch = fixture.batch("op_imported_only");
    batch.operations.push(operation(
        SourceNodeRef::function("timing.veac", "imported"),
        "{ 2500ms }",
    ));
    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert_eq!(preview.changed_modules(), ["timing.veac"]);
    assert_eq!(preview.previous_modules(), ["main.veac", "timing.veac"]);
    assert_eq!(preview.built.sources()["main.veac"], fixture.entry_source);
    assert_eq!(preview.previous_source(), Some(MODULE));
    assert_eq!(
        preview.source().unwrap(),
        MODULE.replace("{ 1s }", "{ 2500ms }")
    );
    assert_eq!(support::result_duration(&preview.built), "3500ms");
    fixture.assert_unchanged();
}

struct Fixture {
    _temp: tempfile::TempDir,
    entry: std::path::PathBuf,
    module: std::path::PathBuf,
    entry_source: String,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let entry = temp.path().join("main.veac");
        let module = temp.path().join("timing.veac");
        let entry_source = entry_source();
        fs::write(&entry, &entry_source).unwrap();
        fs::write(&module, MODULE).unwrap();
        Self {
            _temp: temp,
            entry,
            module,
            entry_source,
        }
    }

    fn batch(&self, id: &str) -> SourceEditBatch {
        let revision = build_path(&self.entry).unwrap().source_revision().unwrap();
        SourceEditBatch::new(veac_ir::OperationId::new(id).unwrap(), revision)
    }

    fn assert_unchanged(&self) {
        assert_eq!(fs::read_to_string(&self.entry).unwrap(), self.entry_source);
        assert_eq!(fs::read_to_string(&self.module).unwrap(), MODULE);
    }
}

fn operation(target: SourceNodeRef, source: &str) -> SourceEditOperation {
    SourceEditOperation::SetBody {
        target,
        site: BodySite::FunctionBody,
        body: BodySource {
            source: source.into(),
        },
    }
}
