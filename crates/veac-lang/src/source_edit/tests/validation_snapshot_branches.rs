use super::*;

struct DefaultSnapshot;

impl SourceSnapshot for DefaultSnapshot {
    fn node_exists(&self, _: &SourceNodeRef) -> bool {
        false
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
}

#[test]
fn source_snapshot_optional_queries_have_closed_absent_defaults() {
    let snapshot = DefaultSnapshot;
    let declaration = SourceNodeRef::function("main.veac", "duration");
    let imported = SourceImportRef::new("main.veac", "timing");
    assert_eq!(snapshot.top_level_declaration_source(&declaration), None);
    assert!(!snapshot.import_exists(&imported));
    assert_eq!(snapshot.import_path(&imported), None);
}

#[test]
fn default_snapshot_methods_drive_structural_preconditions() {
    let snapshot = DefaultSnapshot;
    let cases = [
        SourcePrecondition::TopLevelDeclarationEquals {
            target: SourceNodeRef::function("main.veac", "duration"),
            declaration: TopLevelDeclarationSource {
                source: "fn duration() -> time { 1s }".into(),
            },
        },
        SourcePrecondition::ImportExists {
            target: SourceImportRef::new("main.veac", "timing"),
        },
        SourcePrecondition::ImportEquals {
            target: SourceImportRef::new("main.veac", "timing"),
            import: ImportSource {
                path: "./timing.veac".into(),
                alias: "timing".into(),
            },
        },
    ];
    for precondition in cases {
        let mut value = batch();
        value.preconditions = vec![precondition];
        assert_eq!(
            validate_source_edit_batch(&value, &revision('a'), &snapshot),
            Err(SourceEditError::PreconditionFailed { index: 0 })
        );
    }
}
