use super::support::*;
use veac_lang::program::SourceIndexInventory;
use veac_lang::source_edit::{
    SourceEditBatch, SourceEditOperation, SourceNodeRef, TopLevelDeclarationSource,
};

fn source_text() -> String {
    format!(
        "struct Card {{}}\n\
         impl Card @presentation {{ fn title(self) -> text {{ \"title\" }} }}\n\
         impl Card @timing {{ fn duration(self) -> time {{ 1s }} }}\n{}",
        EXECUTABLE_SOURCE
    )
}

fn implementation(identity: &str) -> SourceNodeRef {
    SourceNodeRef::implementation("main.veac", "Card", identity)
}

#[test]
fn cli_discovers_and_dry_runs_one_of_multiple_named_impl_blocks() {
    let temp = tempdir().unwrap();
    let original = source_text();
    let source = source_file(&temp, &original);
    let indexed = veac()
        .args(["source-index", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        indexed.status.success(),
        "{}",
        String::from_utf8_lossy(&indexed.stderr)
    );
    let inventory: SourceIndexInventory = serde_json::from_slice(&indexed.stdout).unwrap();
    for target in [implementation("presentation"), implementation("timing")] {
        assert!(inventory.modules[0]
            .declarations
            .iter()
            .any(|value| value.target == target));
    }

    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_cli_named_impl").unwrap(),
        inventory.revision,
    );
    batch.operations = vec![SourceEditOperation::SetTopLevelDeclaration {
        target: implementation("presentation"),
        declaration: TopLevelDeclarationSource {
            source: "impl Card @presentation { fn title(self) -> text { \"updated\" } }".into(),
        },
    }];
    let batch_path = temp.path().join("impl-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();
    let edited = veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(
        edited.status.success(),
        "{}",
        String::from_utf8_lossy(&edited.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&edited.stdout).unwrap();
    assert_eq!(report["dry_run"], true);
    assert_eq!(std::fs::read_to_string(source).unwrap(), original);
}
