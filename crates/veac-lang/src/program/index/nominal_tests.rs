use std::collections::BTreeMap;

use crate::source_edit::{
    DeclarationSite, DeclarationSource, SourceEditBatch, SourceEditOperation, SourceNodeRef,
    SourcePrecondition, SourceSnapshot,
};

const SOURCE: &str = r#"module {
  export struct Timing {
    start: time,
    duration: time,
  }
  export enum Placement {
    Center,
    Corner { x: length, y: length, },
  }
}"#;

#[test]
fn indexes_every_nominal_declaration_with_exact_authored_ranges() {
    let index = super::SourceIndex::build_snapshot(&sources()).unwrap();
    let cases = [
        (
            SourceNodeRef::structure("types.veac", "Timing"),
            DeclarationSite::StructDeclaration,
            "struct Timing {\n    start: time,\n    duration: time,\n  }",
        ),
        (
            SourceNodeRef::struct_field("types.veac", "Timing", "start"),
            DeclarationSite::StructFieldDeclaration,
            "start: time",
        ),
        (
            SourceNodeRef::enumeration("types.veac", "Placement"),
            DeclarationSite::EnumDeclaration,
            "enum Placement {\n    Center,\n    Corner { x: length, y: length, },\n  }",
        ),
        (
            SourceNodeRef::enum_variant("types.veac", "Placement", "Corner"),
            DeclarationSite::EnumVariantDeclaration,
            "Corner { x: length, y: length, }",
        ),
        (
            SourceNodeRef::enum_variant_field("types.veac", "Placement", "Corner", "y"),
            DeclarationSite::EnumVariantFieldDeclaration,
            "y: length",
        ),
    ];
    for (target, site, expected) in cases {
        assert!(index.node_exists(&target));
        let declaration = index.declaration(&target, site).unwrap();
        assert_eq!(declaration.source, expected);
        assert_eq!(
            &SOURCE[declaration.range.start..declaration.range.end],
            expected
        );
    }
}

#[test]
fn inventory_and_preconditions_expose_the_same_declaration_source() {
    let index = super::super::test_support::index(&sources());
    let revision = super::super::test_revision(&index);
    let target = SourceNodeRef::enum_variant("types.veac", "Placement", "Center");
    let site = DeclarationSite::EnumVariantDeclaration;
    let inventory = index.inventory(&revision).unwrap();
    let node = inventory
        .nodes
        .iter()
        .find(|node| node.target == target)
        .unwrap();
    assert_eq!(node.declarations.len(), 1);
    assert_eq!(node.declarations[0].source, "Center");
    assert!(node.expressions.is_empty());
    assert!(node.bodies.is_empty());

    let mut batch = SourceEditBatch::new(
        veac_ir::OperationId::new("op_nominal_precondition").unwrap(),
        revision.clone(),
    );
    batch
        .preconditions
        .push(SourcePrecondition::DeclarationEquals {
            target: target.clone(),
            site,
            declaration: DeclarationSource {
                source: "Center".into(),
            },
        });
    batch.operations.push(SourceEditOperation::SetDeclaration {
        target,
        site,
        declaration: DeclarationSource {
            source: "Middle".into(),
        },
    });
    crate::source_edit::validate_source_edit_batch(&batch, &revision, &index).unwrap();
}

fn sources() -> BTreeMap<String, String> {
    BTreeMap::from([("types.veac".into(), SOURCE.into())])
}
