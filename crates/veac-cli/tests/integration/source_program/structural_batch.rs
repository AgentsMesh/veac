use super::*;
use veac_lang::source_edit::{
    BodySite, BodySource, SourceImportRef, SourceModuleAnchor, TopLevelDeclarationSource,
};

const MODULE: &str = "module { export fn old() -> time { 200ms } }\n";

#[test]
fn cli_dry_run_and_commit_report_and_publish_the_same_multi_module_change_set() {
    let fixture = Fixture::new();
    let batch_path = fixture.write_batch();
    let output = veac()
        .args([
            "source-edit",
            fixture.entry.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["modules"],
        serde_json::json!(["main.veac", "timing.veac"])
    );
    assert_eq!(report["destinations"], serde_json::json!([]));
    assert_eq!(report["dry_run"], true);
    fixture.assert_sources(false);
    assert!(fixture.root().join(".veac-source.lock").is_file());

    let output = veac()
        .args([
            "source-edit",
            fixture.entry.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["modules"],
        serde_json::json!(["main.veac", "timing.veac"])
    );
    assert_eq!(report["destinations"].as_array().unwrap().len(), 2);
    fixture.assert_sources(true);
    assert!(fixture.root().join(".veac-source.lock").is_file());
}

#[test]
fn cli_rejects_independent_output_for_a_multi_module_batch() {
    let fixture = Fixture::new();
    let batch_path = fixture.write_batch();
    let output = fixture.root().join("revised.veac");
    veac()
        .args([
            "source-edit",
            fixture.entry.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "SOURCE_EDIT_OUTPUT_REQUIRES_SINGLE_MODULE",
        ));
    assert!(!output.exists());
    fixture.assert_sources(false);
}

struct Fixture {
    _temp: tempfile::TempDir,
    entry: std::path::PathBuf,
    entry_source: String,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let entry_source = format!(
            "import \"./timing.veac\" as timing;\nfn duration() -> time {{ timing.old() }}\n{}",
            GENERATED_SOURCE.replace("during(0s, 200ms)", "during(0s, duration())")
        );
        let entry = source_file(&temp, &entry_source);
        std::fs::write(temp.path().join("timing.veac"), MODULE).unwrap();
        Self {
            _temp: temp,
            entry,
            entry_source,
        }
    }

    fn root(&self) -> &std::path::Path {
        self._temp.path()
    }

    fn write_batch(&self) -> std::path::PathBuf {
        let prepared = veac_lang::program::prepare_path(&self.entry).unwrap();
        let mut batch = SourceEditBatch::new(
            veac_ir::OperationId::new("op_cli_structural_batch").unwrap(),
            prepared.source_index().unwrap().revision().clone(),
        );
        batch.operations = vec![
            SourceEditOperation::RemoveImport {
                target: SourceImportRef::new("main.veac", "timing"),
            },
            SourceEditOperation::InsertImport {
                module: "main.veac".into(),
                anchor: SourceModuleAnchor::ModuleStart,
                import: veac_lang::source_edit::ImportSource {
                    path: "./timing.veac".into(),
                    alias: "renamed".into(),
                },
            },
            SourceEditOperation::SetBody {
                target: SourceNodeRef::function("main.veac", "duration"),
                site: BodySite::FunctionBody,
                body: BodySource {
                    source: "{ renamed.fresh() }".into(),
                },
            },
            SourceEditOperation::RemoveDeclaration {
                target: SourceNodeRef::function("timing.veac", "old"),
            },
            SourceEditOperation::InsertDeclaration {
                module: "timing.veac".into(),
                anchor: SourceModuleAnchor::ModuleEnd,
                declaration: TopLevelDeclarationSource {
                    source: "export fn fresh() -> time { 400ms }".into(),
                },
            },
        ];
        let path = self.root().join("edit.json");
        std::fs::write(
            &path,
            veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
        )
        .unwrap();
        path
    }

    fn assert_sources(&self, edited: bool) {
        let entry = std::fs::read_to_string(&self.entry).unwrap();
        let module = std::fs::read_to_string(self.root().join("timing.veac")).unwrap();
        if edited {
            assert!(entry.contains("as renamed"));
            assert!(entry.contains("renamed.fresh()"));
            assert!(module.contains("fn fresh"));
            assert!(!module.contains("fn old"));
        } else {
            assert_eq!(entry, self.entry_source);
            assert_eq!(module, MODULE);
        }
    }
}
