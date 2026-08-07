use veac_lang::program::apply_executable_source_edit_path;
use veac_lang::source_edit::{SourceEditOperation, SourceModuleAnchor, SourceNodeRef};

use super::support::{declaration, duration, project_with, Fixture};

fn implementation(identity: &str) -> SourceNodeRef {
    SourceNodeRef::implementation("main.veac", "Timing", identity)
}

#[test]
fn named_impl_blocks_support_exact_set_remove_and_typed_anchor_edits() {
    let declarations = r#"struct Timing {}
impl Timing @base { fn base(self) -> time { 1s } }
impl Timing @padding { fn padding(self) -> time { 250ms } }
impl Timing @tail { fn tail(self) -> time { 500ms } }"#;
    let fixture = Fixture::new(&project_with(declarations, "1s"), &[]);
    let mut batch = fixture.batch("op_named_impl_blocks");
    batch.operations = vec![
        SourceEditOperation::SetTopLevelDeclaration {
            target: implementation("base"),
            declaration: declaration("impl Timing @base { fn base(self) -> time { 2s } }"),
        },
        SourceEditOperation::RemoveDeclaration {
            target: implementation("padding"),
        },
        SourceEditOperation::InsertDeclaration {
            module: "main.veac".into(),
            anchor: SourceModuleAnchor::BeforeDeclaration {
                target: implementation("tail"),
            },
            declaration: declaration("impl Timing @extra { fn extra(self) -> time { 750ms } }"),
        },
    ];

    let preview = apply_executable_source_edit_path(&fixture.entry, &batch).unwrap();
    let edited = preview.source().unwrap();
    assert!(edited.contains("@base { fn base(self) -> time { 2s } }"));
    assert!(!edited.contains("@padding"));
    assert!(edited.find("@extra").unwrap() < edited.find("@tail").unwrap());
    assert_eq!(duration(&preview.built), 1_000);
    fixture.assert_unchanged();
}
