use super::support::*;
use veac_lang::program::{SourceIndexInventory, SOURCE_INDEX_SCHEMA_VERSION};
use veac_lang::source_edit::{
    BodySite, BodySource, DeclarationSite, DeclarationSource, ExpressionSite, ExpressionSource,
    SourceEditBatch, SourceEditOperation, SourceNodeRef,
};

fn nominal_source() -> String {
    let mut source = r#"struct Timing { duration: time, }
enum CardPlacement { Center, Corner { x: length, }, }
fn read(value: Timing) -> time { value.duration }
const Timing timing = Timing { duration: 250ms, };
"#
    .to_owned();
    source.push_str(EXECUTABLE_SOURCE);
    source
}

#[test]
fn source_index_command_emits_nominal_declaration_inventory_v8() {
    let temp = tempdir().unwrap();
    let source_text = nominal_source();
    let source = source_file(&temp, &source_text);
    let output = veac()
        .args(["source-index", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let inventory: SourceIndexInventory = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(inventory.schema_version, SOURCE_INDEX_SCHEMA_VERSION);
    let target = SourceNodeRef::enum_variant_field("main.veac", "CardPlacement", "Corner", "x");
    let node = inventory
        .nodes
        .iter()
        .find(|node| node.target == target)
        .unwrap();
    assert_eq!(
        node.declarations[0].site,
        DeclarationSite::EnumVariantFieldDeclaration
    );
    assert_eq!(node.declarations[0].source, "x: length");
    let range = node.declarations[0].range;
    assert_eq!(&source_text[range.start..range.end], "x: length");
}

#[test]
fn source_edit_command_atomically_applies_a_nominal_field_rename() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &nominal_source());
    let revision = revision(&source);
    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_cli_nominal_declaration").unwrap(),
        revision,
    );
    batch.operations.push(SourceEditOperation::SetDeclaration {
        target: SourceNodeRef::struct_field("main.veac", "Timing", "duration"),
        site: DeclarationSite::StructFieldDeclaration,
        declaration: DeclarationSource {
            source: "amount: time".into(),
        },
    });
    batch.operations.push(SourceEditOperation::SetBody {
        target: SourceNodeRef::function("main.veac", "read"),
        site: BodySite::FunctionBody,
        body: BodySource {
            source: "{ value.amount }".into(),
        },
    });
    batch.operations.push(SourceEditOperation::SetExpression {
        target: SourceNodeRef::constant("main.veac", "timing"),
        site: ExpressionSite::ConstantValue,
        expression: ExpressionSource {
            source: "Timing { amount: 250ms, }".into(),
        },
    });
    let batch_path = temp.path().join("nominal-edit.json");
    std::fs::write(
        &batch_path,
        veac_lang::source_edit::canonical_source_edit_batch_json(&batch).unwrap(),
    )
    .unwrap();

    let output = veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["modules"], serde_json::json!(["main.veac"]));
    let edited = std::fs::read_to_string(source).unwrap();
    assert!(edited.contains("amount: time"));
    assert!(edited.contains("value.amount"));
    assert!(edited.contains("Timing { amount: 250ms, }"));
}

fn revision(source: &std::path::Path) -> veac_lang::source_edit::SourceRevision {
    let output = veac()
        .args(["source-revision", source.to_str().unwrap()])
        .output()
        .unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}
