use veac_lang::program::{apply_executable_source_edit_path, SourceTransactionError};
use veac_lang::source_edit::{
    SourceEditError, SourceEditOperation, SourceModuleAnchor, SourceNodeRef,
};

use super::support::{declaration, project_with, Fixture};

#[test]
fn missing_typed_anchor_reports_the_operation_and_writes_nothing() {
    let entry = project_with("fn duration() -> time { 1s }", "duration()");
    let fixture = Fixture::new(&entry, &[]);
    let mut batch = fixture.batch("op_missing_structural_anchor");
    batch.operations = vec![SourceEditOperation::InsertDeclaration {
        module: "main.veac".into(),
        anchor: SourceModuleAnchor::BeforeDeclaration {
            target: SourceNodeRef::function("main.veac", "missing"),
        },
        declaration: declaration("fn inserted() -> time { 2s }"),
    }];
    let error = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap_err();
    assert!(matches!(
        &error,
        SourceTransactionError::TargetNotFound { operation: 0 }
    ));
    assert!(error.to_string().contains("operation 0"));
    fixture.assert_unchanged();
}

#[test]
fn duplicate_zero_width_insertions_are_rejected_as_ambiguous() {
    let entry = project_with("fn duration() -> time { 1s }", "duration()");
    let fixture = Fixture::new(&entry, &[]);
    let mut batch = fixture.batch("op_duplicate_structural_anchor");
    batch.operations = vec![
        SourceEditOperation::InsertDeclaration {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::ModuleEnd,
            declaration: declaration("fn first() -> time { 1s }"),
        },
        SourceEditOperation::InsertDeclaration {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::ModuleEnd,
            declaration: declaration("fn second() -> time { 2s }"),
        },
    ];
    assert!(matches!(
        apply_executable_source_edit_path(&fixture.entry, &batch),
        Err(SourceTransactionError::Contract(
            SourceEditError::OverlappingTextEdits
        ))
    ));
    fixture.assert_unchanged();
}

#[test]
fn root_owned_declarations_cannot_be_injected_into_imported_modules() {
    let entry = project_with(
        "import \"./timing.veac\" as timing;\nfn duration() -> time { timing.duration() }",
        "duration()",
    );
    let module = "module { export fn duration() -> time { 1s } }\n";
    for (id, source) in [
        ("op_module_input_injection", "input parameter injected: time;"),
        (
            "op_module_temporal_injection",
            "animate visual-opacity on clip(@structural-edit, @main, @visual, @result) { progress }",
        ),
    ] {
        let fixture = Fixture::new(&entry, &[("timing.veac", module)]);
        let mut batch = fixture.batch(id);
        batch.operations = vec![SourceEditOperation::InsertDeclaration {
            module: "timing.veac".into(),
            anchor: SourceModuleAnchor::ModuleStart,
            declaration: declaration(source),
        }];
        assert!(matches!(
            apply_executable_source_edit_path(&fixture.entry, &batch),
            Err(SourceTransactionError::Program(_))
        ));
        fixture.assert_unchanged();
    }
}

#[test]
fn changed_module_must_remain_reachable_in_the_validated_candidate() {
    let entry = project_with(
        "import \"./timing.veac\" as timing;\nfn duration() -> time { 1s }",
        "duration()",
    );
    let module = "module { export fn unused() -> time { 1s } }\n";
    let fixture = Fixture::new(&entry, &[("timing.veac", module)]);
    let mut batch = fixture.batch("op_unreachable_changed_module");
    batch.operations = vec![
        SourceEditOperation::RemoveImport {
            target: veac_lang::source_edit::SourceImportRef::new("main.veac", "timing"),
        },
        SourceEditOperation::SetTopLevelDeclaration {
            target: SourceNodeRef::function("timing.veac", "unused"),
            declaration: declaration("export fn unused() -> time { 2s }"),
        },
    ];
    let error = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap_err();
    assert!(matches!(
        &error,
        SourceTransactionError::ChangedModuleUnreachable { module }
            if module == "timing.veac"
    ));
    assert!(error.to_string().contains("unreachable"));
    fixture.assert_unchanged();
}
