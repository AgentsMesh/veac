use std::fs;

use veac_lang::program::{
    prepare_executable_source_edit_with_loader, prepare_with_loader,
    reprepare_executable_source_edit_preview, BuildInputManifestV1, FileSystemLoader,
    SourceTransactionError,
};
use veac_lang::source_edit::{
    BodySite, BodySource, ImportSource, SourceEditBatch, SourceEditOperation, SourceModuleAnchor,
    SourceNodeRef,
};

#[path = "program_functions/support.rs"]
mod support;

const FRESH: &str = "module { export fn duration() -> time { 2500ms } }\n";
const DRIFTED: &str = "module { export fn duration() -> time { 3s } }\n";

#[test]
fn preview_captures_and_reprepares_the_complete_fresh_project_graph() {
    let temp = tempfile::tempdir().unwrap();
    let entry = write_project(&temp, "fn duration() -> time { 1s }");
    fs::write(temp.path().join("fresh.veac"), FRESH).unwrap();
    let (loader, root) = FileSystemLoader::for_entry(&entry).unwrap();
    let prepared = prepare_with_loader(root, &loader).unwrap();
    let mut batch = batch(&prepared, "op_fresh_project_graph");
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
    let preview = prepare_executable_source_edit_with_loader(&prepared, &batch, &loader)
        .unwrap()
        .execute(&BuildInputManifestV1::empty())
        .unwrap();
    assert_eq!(
        preview.candidate_sources().keys().collect::<Vec<_>>(),
        ["fresh.veac", "main.veac"]
    );
    assert_eq!(support::result_duration(&preview.built), "2500ms");

    let (stable, stable_entry) = FileSystemLoader::for_entry(&entry).unwrap();
    let rebuilt =
        reprepare_executable_source_edit_preview(&preview, stable_entry, &stable).unwrap();
    assert_eq!(rebuilt.source_graph(), preview.built.source_graph());
    assert_eq!(rebuilt.source_revision().unwrap(), preview.new_revision);

    fs::write(temp.path().join("fresh.veac"), DRIFTED).unwrap();
    let (changed, changed_entry) = FileSystemLoader::for_entry(&entry).unwrap();
    let rebuilt =
        reprepare_executable_source_edit_preview(&preview, changed_entry, &changed).unwrap();
    assert_ne!(rebuilt.source_graph(), preview.built.source_graph());
    assert_ne!(rebuilt.source_revision().unwrap(), preview.new_revision);
}

#[test]
fn candidate_reloads_and_rejects_drifted_unchanged_project_dependencies() {
    let temp = tempfile::tempdir().unwrap();
    let declaration =
        "import \"./fresh.veac\" as fresh;\nfn duration() -> time { fresh.duration() }";
    let entry = write_project(&temp, declaration);
    let dependency = temp.path().join("fresh.veac");
    fs::write(&dependency, FRESH).unwrap();
    let (loader, root) = FileSystemLoader::for_entry(&entry).unwrap();
    let prepared = prepare_with_loader(root, &loader).unwrap();
    let mut edit = batch(&prepared, "op_project_dependency_drift");
    edit.operations
        .push(set_duration("{ fresh.duration() + 1s }"));

    fs::write(dependency, DRIFTED).unwrap();
    let (changed, _) = FileSystemLoader::for_entry(&entry).unwrap();
    let SourceTransactionError::Program(diagnostics) =
        prepare_executable_source_edit_with_loader(&prepared, &edit, &changed).unwrap_err()
    else {
        panic!("unchanged project dependency drift must fail preparation");
    };
    assert_eq!(diagnostics.as_slice()[0].code, "PROGRAM_IMPORT_LOAD");
    assert!(diagnostics.as_slice()[0].message.contains("changed bytes"));
}

#[test]
fn reprepare_observes_an_unedited_root_drift() {
    let temp = tempfile::tempdir().unwrap();
    let declaration =
        "import \"./fresh.veac\" as fresh;\nfn duration() -> time { fresh.duration() }";
    let entry = write_project(&temp, declaration);
    fs::write(temp.path().join("fresh.veac"), FRESH).unwrap();
    let (loader, root) = FileSystemLoader::for_entry(&entry).unwrap();
    let prepared = prepare_with_loader(root, &loader).unwrap();
    let mut edit = batch(&prepared, "op_unedited_root_drift");
    edit.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("fresh.veac", "duration"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ 2600ms }".into(),
        },
    });
    let preview = prepare_executable_source_edit_with_loader(&prepared, &edit, &loader)
        .unwrap()
        .execute(&BuildInputManifestV1::empty())
        .unwrap();

    let original = fs::read_to_string(&entry).unwrap();
    fs::write(&entry, format!("\n{original}")).unwrap();
    let (current, current_entry) = FileSystemLoader::for_entry(&entry).unwrap();
    let rebuilt =
        reprepare_executable_source_edit_preview(&preview, current_entry, &current).unwrap();
    assert_ne!(rebuilt.source_graph(), preview.built.source_graph());
    assert_ne!(rebuilt.source_revision().unwrap(), preview.new_revision);
}

fn write_project(temp: &tempfile::TempDir, declarations: &str) -> std::path::PathBuf {
    let entry = temp.path().join("main.veac");
    fs::write(&entry, support::project_with(declarations, "duration()")).unwrap();
    entry
}

fn batch(prepared: &veac_lang::program::ExecutableBuild, operation: &str) -> SourceEditBatch {
    SourceEditBatch::new(
        veac_ir::OperationId::new(operation).unwrap(),
        prepared.source_revision().unwrap(),
    )
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
