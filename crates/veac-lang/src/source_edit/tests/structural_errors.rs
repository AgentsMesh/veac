use super::*;

#[test]
fn import_contract_rejects_invalid_paths_aliases_and_targets() {
    for (path, alias) in [
        ("", "timing"),
        ("./bad\0.veac", "timing"),
        ("./timing.veac", "bad.alias"),
    ] {
        let mut value = batch();
        value.operations = vec![SourceEditOperation::InsertImport {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::ModuleStart,
            import: ImportSource {
                path: path.into(),
                alias: alias.into(),
            },
        }];
        assert!(matches!(
            validate_source_edit_contract(&value),
            Err(SourceEditError::InvalidImport(_))
        ));
    }

    let mut value = batch();
    value.operations = vec![SourceEditOperation::RemoveImport {
        target: SourceImportRef::new("../main.veac", "timing"),
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidModulePath(_))
    ));
}

#[test]
fn import_anchor_cannot_cross_module_boundaries() {
    let mut value = batch();
    value.operations = vec![SourceEditOperation::InsertImport {
        module: "main.veac".into(),
        anchor: SourceModuleAnchor::AfterImport {
            target: SourceImportRef::new("other.veac", "timing"),
        },
        import: ImportSource {
            path: "./timing.veac".into(),
            alias: "timing".into(),
        },
    }];
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::AnchorModuleMismatch {
            module: "main.veac".into(),
            anchor_module: "other.veac".into(),
        })
    );
}

#[test]
fn oversized_import_path_and_non_top_level_target_fail_closed() {
    let mut value = batch();
    value.operations = vec![SourceEditOperation::InsertImport {
        module: "main.veac".into(),
        anchor: SourceModuleAnchor::ModuleEnd,
        import: ImportSource {
            path: "x".repeat(4_097),
            alias: "timing".into(),
        },
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidImport(_))
    ));

    value.operations = vec![SourceEditOperation::SetTopLevelDeclaration {
        target: SourceNodeRef::method("main.veac", "Timing", "duration"),
        declaration: TopLevelDeclarationSource {
            source: "fn duration() -> time { 1s }".into(),
        },
    }];
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::IncompatibleTopLevelDeclarationTarget)
    );
}

#[test]
fn range_only_resolver_refuses_structural_operations_without_an_index() {
    let operation = SourceEditOperation::RemoveDeclaration {
        target: SourceNodeRef::function("main.veac", "duration"),
    };
    assert_eq!(
        resolve_source_edit_text(0, &operation, TextRange { start: 0, end: 1 }),
        Err(SourceEditError::StructuralOperationRequiresIndex)
    );
}
