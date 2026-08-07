use veac_lang::program::apply_executable_source_edit_path;
use veac_lang::source_edit::{
    SourceEditOperation, SourceImportRef, SourceModuleAnchor, SourceNodeRef,
};

use super::support::{declaration, import, project_with, Fixture};

#[test]
fn inserts_resolve_before_and_after_typed_import_and_declaration_anchors() {
    let entry = project_with(
        "import \"./a.veac\" as a;\nfn duration() -> time { 1s }",
        "duration()",
    );
    let fixture = Fixture::new(
        &entry,
        &[
            ("a.veac", "module {}\n"),
            ("b.veac", "module {}\n"),
            ("c.veac", "module {}\n"),
        ],
    );
    let mut batch = fixture.batch("op_all_typed_anchors");
    let imported = SourceImportRef::new("main.veac", "a");
    let function = SourceNodeRef::function("main.veac", "duration");
    batch.operations = vec![
        SourceEditOperation::InsertImport {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::BeforeImport {
                target: imported.clone(),
            },
            import: import("./b.veac", "b"),
        },
        SourceEditOperation::InsertImport {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::AfterImport { target: imported },
            import: import("./c.veac", "c"),
        },
        SourceEditOperation::InsertDeclaration {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::BeforeDeclaration {
                target: function.clone(),
            },
            declaration: declaration("fn before() -> time { 2s }"),
        },
        SourceEditOperation::InsertDeclaration {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::AfterDeclaration { target: function },
            declaration: declaration("fn after() -> time { 3s }"),
        },
    ];
    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    let source = preview.source().unwrap();
    assert!(ordered(source, "as b", "as a"));
    assert!(ordered(source, "as a", "as c"));
    assert!(ordered(source, "fn before", "fn duration"));
    assert!(ordered(source, "fn duration", "fn after"));
    for module in ["a.veac", "b.veac", "c.veac"] {
        assert!(preview.built.sources().contains_key(module));
    }
    fixture.assert_unchanged();
}

fn ordered(source: &str, left: &str, right: &str) -> bool {
    source.find(left).unwrap() < source.find(right).unwrap()
}
