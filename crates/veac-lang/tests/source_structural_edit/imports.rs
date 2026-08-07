use veac_lang::program::{apply_executable_source_edit_path, SourceTransactionError};
use veac_lang::source_edit::{
    BodySite, BodySource, SourceEditOperation, SourceImportRef, SourceModuleAnchor, SourceNodeRef,
    SourcePrecondition,
};

use super::support::{declaration, duration, import, project_with, Fixture};

const TIMING: &str = "module {\n  export fn old() -> time { 1s }\n}\n";

#[test]
fn one_batch_renames_an_imported_api_across_both_modules() {
    let entry = project_with(
        "import \"./timing.veac\" as timing;\nfn duration() -> time { timing.old() }",
        "duration()",
    );
    let fixture = Fixture::new(&entry, &[("timing.veac", TIMING)]);
    let mut batch = fixture.batch("op_structural_module_rename");
    let old_import = SourceImportRef::new("main.veac", "timing");
    let old_function = SourceNodeRef::function("timing.veac", "old");
    batch.preconditions = vec![
        SourcePrecondition::ImportEquals {
            target: old_import.clone(),
            import: import("./timing.veac", "timing"),
        },
        SourcePrecondition::TopLevelDeclarationEquals {
            target: old_function.clone(),
            declaration: declaration("export fn old() -> time { 1s }"),
        },
    ];
    batch.operations = vec![
        SourceEditOperation::RemoveImport { target: old_import },
        SourceEditOperation::InsertImport {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::ModuleStart,
            import: import("./timing.veac", "renamed"),
        },
        SourceEditOperation::SetBody {
            target: SourceNodeRef::function("main.veac", "duration"),
            site: BodySite::FunctionBody,
            body: BodySource {
                source: "{ renamed.fresh() }".into(),
            },
        },
        SourceEditOperation::RemoveDeclaration {
            target: old_function,
        },
        SourceEditOperation::InsertDeclaration {
            module: "timing.veac".into(),
            anchor: SourceModuleAnchor::ModuleEnd,
            declaration: declaration("export fn fresh() -> time { 2s }"),
        },
    ];

    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert_eq!(preview.changed_modules(), ["main.veac", "timing.veac"]);
    assert_eq!(duration(&preview.built), 2_000);
    assert!(preview.changes()[0].source().contains("as renamed"));
    assert!(preview.changes()[1].source().contains("fn fresh"));
    assert!(!preview.changes()[1].source().contains("fn old"));
    fixture.assert_unchanged();
}

#[test]
fn inserted_import_loads_a_preexisting_module_outside_the_old_graph() {
    let entry = project_with("fn duration() -> time { 1s }", "duration()");
    let fresh = "module { export fn duration() -> time { 2500ms } }\n";
    let fixture = Fixture::new(&entry, &[("fresh.veac", fresh)]);
    let mut batch = fixture.batch("op_load_inserted_import");
    batch.operations = vec![
        SourceEditOperation::InsertImport {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::ModuleStart,
            import: import("./fresh.veac", "fresh"),
        },
        SourceEditOperation::SetBody {
            target: SourceNodeRef::function("main.veac", "duration"),
            site: BodySite::FunctionBody,
            body: BodySource {
                source: "{ fresh.duration() }".into(),
            },
        },
    ];
    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    assert_eq!(preview.previous_modules(), ["main.veac"]);
    assert!(preview.built.sources().contains_key("fresh.veac"));
    assert_eq!(duration(&preview.built), 2_500);
    fixture.assert_unchanged();
}

#[test]
fn wrong_import_path_fails_the_typed_precondition_before_selection() {
    let entry = project_with(
        "import \"./timing.veac\" as timing;\nfn duration() -> time { timing.old() }",
        "duration()",
    );
    let fixture = Fixture::new(&entry, &[("timing.veac", TIMING)]);
    let mut batch = fixture.batch("op_wrong_import_precondition");
    batch.preconditions.push(SourcePrecondition::ImportEquals {
        target: SourceImportRef::new("main.veac", "timing"),
        import: import("./other.veac", "timing"),
    });
    batch.operations.push(SourceEditOperation::RemoveImport {
        target: SourceImportRef::new("main.veac", "timing"),
    });
    assert!(matches!(
        apply_executable_source_edit_path(&fixture.entry, &batch),
        Err(SourceTransactionError::Contract(
            veac_lang::source_edit::SourceEditError::PreconditionFailed { index: 0 }
        ))
    ));
    fixture.assert_unchanged();
}
