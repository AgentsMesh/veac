use super::{commit_in_place, source_changed};
use crate::unit_tests::support::EXECUTABLE_SOURCE;
use veac_lang::source_edit::{
    BodySite, BodySource, ImportSource, SourceEditBatch, SourceEditOperation, SourceImportRef,
    SourceModuleAnchor, SourceNodeRef,
};

const MODULE: &str = "module { export fn duration() -> time { 400ms } }\n";

#[test]
fn source_changed_keeps_the_candidate_module_in_the_diagnostic() {
    let error = source_changed(std::path::Path::new("/project"), "main.veac");
    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert!(error.diagnostics()[0]
        .message
        .contains("/project/main.veac"));
}

#[test]
fn newly_imported_project_dependency_drift_prevents_publication() {
    let fixture = Fixture::new("fn duration() -> time { 200ms }\n");
    let fresh = fixture.root().join("fresh.veac");
    std::fs::write(&fresh, MODULE).unwrap();
    let mut batch = fixture.batch("op_cli_fresh_dependency");
    batch.operations = vec![
        SourceEditOperation::InsertImport {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::ModuleStart,
            import: ImportSource {
                path: "./fresh.veac".into(),
                alias: "fresh".into(),
            },
        },
        set_duration("{ fresh.duration() }"),
    ];
    let preview =
        veac_lang::program::apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert!(!preview.previous_sources().contains_key("fresh.veac"));
    assert!(preview.candidate_sources().contains_key("fresh.veac"));

    std::fs::write(&fresh, MODULE.replace("400ms", "600ms")).unwrap();
    let error = commit_in_place(fixture.root(), &[], &preview).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    fixture.assert_entry_unchanged();
}

#[test]
fn removed_project_dependency_drift_prevents_publication() {
    let declarations = "import \"./old.veac\" as old;\nfn duration() -> time { old.duration() }\n";
    let fixture = Fixture::new(declarations);
    let old = fixture.root().join("old.veac");
    std::fs::write(&old, MODULE).unwrap();
    let mut batch = fixture.batch("op_cli_removed_dependency");
    batch.operations = vec![
        SourceEditOperation::RemoveImport {
            target: SourceImportRef::new("main.veac", "old"),
        },
        set_duration("{ 300ms }"),
    ];
    let preview =
        veac_lang::program::apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert!(preview.previous_sources().contains_key("old.veac"));
    assert!(!preview.candidate_sources().contains_key("old.veac"));

    std::fs::write(&old, MODULE.replace("400ms", "800ms")).unwrap();
    let error = commit_in_place(fixture.root(), &[], &preview).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    fixture.assert_entry_unchanged();
}

fn set_duration(source: &str) -> SourceEditOperation {
    SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "duration"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: source.into(),
        },
    }
}

struct Fixture {
    _temp: tempfile::TempDir,
    entry: std::path::PathBuf,
    source: String,
}

impl Fixture {
    fn new(declarations: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let source = format!(
            "{declarations}{}",
            EXECUTABLE_SOURCE.replace("during(0s, 200ms)", "during(0s, duration())")
        );
        let entry = temp.path().join("main.veac");
        std::fs::write(&entry, &source).unwrap();
        Self {
            _temp: temp,
            entry,
            source,
        }
    }

    fn root(&self) -> &std::path::Path {
        self._temp.path()
    }

    fn batch(&self, id: &str) -> SourceEditBatch {
        let prepared = veac_lang::program::prepare_path(&self.entry).unwrap();
        SourceEditBatch::new(
            veac_ir::OperationId::new(id).unwrap(),
            prepared.source_revision().unwrap(),
        )
    }

    fn assert_entry_unchanged(&self) {
        assert_eq!(std::fs::read_to_string(&self.entry).unwrap(), self.source);
    }
}
