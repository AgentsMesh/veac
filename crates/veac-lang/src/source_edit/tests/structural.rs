use std::collections::BTreeMap;

use super::*;

#[derive(Default)]
struct Snapshot {
    declarations: BTreeMap<SourceNodeRef, String>,
    imports: BTreeMap<SourceImportRef, String>,
}

impl SourceSnapshot for Snapshot {
    fn node_exists(&self, target: &SourceNodeRef) -> bool {
        self.declarations.contains_key(target)
    }

    fn expression_source(&self, _: &SourceNodeRef, _: &ExpressionSite) -> Option<&str> {
        None
    }

    fn statement_source(&self, _: &SourceNodeRef, _: &StatementSite) -> Option<&str> {
        None
    }

    fn body_source(&self, _: &SourceNodeRef, _: BodySite) -> Option<&str> {
        None
    }

    fn declaration_source(&self, _: &SourceNodeRef, _: DeclarationSite) -> Option<&str> {
        None
    }

    fn top_level_declaration_source(&self, target: &SourceNodeRef) -> Option<&str> {
        self.declarations.get(target).map(String::as_str)
    }

    fn import_exists(&self, target: &SourceImportRef) -> bool {
        self.imports.contains_key(target)
    }

    fn import_path(&self, target: &SourceImportRef) -> Option<&str> {
        self.imports.get(target).map(String::as_str)
    }
}

#[test]
fn structural_contract_accepts_only_typed_fragments_and_targets() {
    let mut value = batch();
    value.operations = vec![
        SourceEditOperation::SetTopLevelDeclaration {
            target: SourceNodeRef::function("timeline/main.veac", "duration"),
            declaration: declaration("fn duration() -> time { 2s }"),
        },
        SourceEditOperation::InsertDeclaration {
            module: "timeline/main.veac".into(),
            anchor: SourceModuleAnchor::ModuleEnd,
            declaration: declaration("const time padding = 1s;"),
        },
        SourceEditOperation::RemoveDeclaration {
            target: SourceNodeRef::enumeration("timeline/main.veac", "Unused"),
        },
        SourceEditOperation::InsertImport {
            module: "timeline/main.veac".into(),
            anchor: SourceModuleAnchor::ModuleStart,
            import: import("./timing.veac", "timing"),
        },
        SourceEditOperation::RemoveImport {
            target: SourceImportRef::new("timeline/main.veac", "old"),
        },
    ];
    assert_eq!(validate_source_edit_contract(&value), Ok(()));
}

#[test]
fn structural_preconditions_compare_exact_declarations_and_imports() {
    let target = SourceNodeRef::function("timeline/main.veac", "duration");
    let imported = SourceImportRef::new("timeline/main.veac", "timing");
    let absent = SourceImportRef::new("timeline/main.veac", "unused");
    let source = "fn duration() -> time { 1s }";
    let mut snapshot = Snapshot::default();
    snapshot.declarations.insert(target.clone(), source.into());
    snapshot
        .imports
        .insert(imported.clone(), "./timing.veac".into());
    let mut value = batch();
    value.preconditions = vec![
        SourcePrecondition::TopLevelDeclarationEquals {
            target,
            declaration: declaration(source),
        },
        SourcePrecondition::ImportExists {
            target: imported.clone(),
        },
        SourcePrecondition::ImportEquals {
            target: imported,
            import: import("./timing.veac", "timing"),
        },
        SourcePrecondition::ImportAbsent { target: absent },
    ];
    assert_eq!(
        validate_source_edit_batch(&value, &revision('a'), &snapshot),
        Ok(())
    );
    let SourcePrecondition::ImportEquals { import, .. } = &mut value.preconditions[2] else {
        unreachable!()
    };
    import.path = "./other.veac".into();
    assert_eq!(
        validate_source_edit_batch(&value, &revision('a'), &snapshot),
        Err(SourceEditError::PreconditionFailed { index: 2 })
    );
}

#[test]
fn structural_contract_rejects_mismatched_anchors_aliases_and_kinds() {
    let mut value = batch();
    value.operations = vec![SourceEditOperation::InsertDeclaration {
        module: "main.veac".into(),
        anchor: SourceModuleAnchor::BeforeDeclaration {
            target: SourceNodeRef::function("other.veac", "duration"),
        },
        declaration: declaration("fn inserted() -> time { 1s }"),
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::AnchorModuleMismatch { .. })
    ));

    value.operations = vec![SourceEditOperation::RemoveDeclaration {
        target: SourceNodeRef::item("main.veac", "demo", "main", "visual", "clip"),
    }];
    assert_eq!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::IncompatibleTopLevelDeclarationTarget)
    );

    value.operations = vec![operation("1s")];
    value.preconditions = vec![SourcePrecondition::ImportEquals {
        target: SourceImportRef::new("main.veac", "timing"),
        import: import("./timing.veac", "other"),
    }];
    assert!(matches!(
        validate_source_edit_contract(&value),
        Err(SourceEditError::InvalidImport(_))
    ));
}

#[test]
fn declaration_fragments_reject_wrappers_imports_multiple_items_and_invalid_syntax() {
    for source in [
        "module { fn value() -> time { 1s } }",
        "import \"./value.veac\" as value;",
        "fn first() -> time { 1s } fn second() -> time { 2s }",
        "fn broken(",
    ] {
        let mut value = batch();
        value.operations = vec![SourceEditOperation::InsertDeclaration {
            module: "timeline/main.veac".into(),
            anchor: SourceModuleAnchor::ModuleEnd,
            declaration: declaration(source),
        }];
        assert!(matches!(
            validate_source_edit_contract(&value),
            Err(SourceEditError::InvalidTopLevelDeclaration(_))
        ));
    }
}

fn declaration(source: &str) -> TopLevelDeclarationSource {
    TopLevelDeclarationSource {
        source: source.into(),
    }
}

fn import(path: &str, alias: &str) -> ImportSource {
    ImportSource {
        path: path.into(),
        alias: alias.into(),
    }
}
