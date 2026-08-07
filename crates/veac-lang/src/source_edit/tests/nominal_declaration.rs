use super::*;

#[test]
fn nominal_declaration_targets_have_closed_module_qualified_shapes() {
    let target =
        SourceNodeRef::enum_variant_field("types/placement.veac", "Placement", "Corner", "x");
    assert_eq!(
        serde_json::to_value(&target).unwrap(),
        serde_json::json!({
            "module": "types/placement.veac",
            "path": {
                "kind": "enum_variant_field",
                "enumeration": "Placement",
                "variant": "Corner",
                "field": "x"
            }
        })
    );
    assert_eq!(target.kind(), SourceNodeKind::EnumVariantField);
    assert_eq!(
        serde_json::from_value::<SourceNodeRef>(serde_json::to_value(&target).unwrap()).unwrap(),
        target
    );
}

#[test]
fn each_declaration_site_accepts_only_its_nominal_node_kind() {
    let cases = [
        (
            DeclarationSite::StructDeclaration,
            SourceNodeKind::Struct,
            SourceNodeRef::structure("types.veac", "Timing"),
        ),
        (
            DeclarationSite::StructFieldDeclaration,
            SourceNodeKind::StructField,
            SourceNodeRef::struct_field("types.veac", "Timing", "duration"),
        ),
        (
            DeclarationSite::EnumDeclaration,
            SourceNodeKind::Enum,
            SourceNodeRef::enumeration("types.veac", "Placement"),
        ),
        (
            DeclarationSite::EnumVariantDeclaration,
            SourceNodeKind::EnumVariant,
            SourceNodeRef::enum_variant("types.veac", "Placement", "Center"),
        ),
        (
            DeclarationSite::EnumVariantFieldDeclaration,
            SourceNodeKind::EnumVariantField,
            SourceNodeRef::enum_variant_field("types.veac", "Placement", "Corner", "x"),
        ),
    ];
    for (site, kind, target) in cases {
        assert!(site.accepts(kind));
        assert!(site.accepts_target(&target));
        assert!(!site.accepts(SourceNodeKind::Function));
    }
}

#[test]
fn contract_parses_each_typed_nominal_declaration_fragment() {
    let cases = [
        (
            SourceNodeRef::structure("types.veac", "Timing"),
            DeclarationSite::StructDeclaration,
            "struct Timing { duration: time, }",
        ),
        (
            SourceNodeRef::struct_field("types.veac", "Timing", "duration"),
            DeclarationSite::StructFieldDeclaration,
            "duration: time",
        ),
        (
            SourceNodeRef::enumeration("types.veac", "Placement"),
            DeclarationSite::EnumDeclaration,
            "enum Placement { Center, Corner { x: length, }, }",
        ),
        (
            SourceNodeRef::enum_variant("types.veac", "Placement", "Corner"),
            DeclarationSite::EnumVariantDeclaration,
            "Corner { x: length, }",
        ),
        (
            SourceNodeRef::enum_variant_field("types.veac", "Placement", "Corner", "x"),
            DeclarationSite::EnumVariantFieldDeclaration,
            "x: length",
        ),
    ];
    for (target, site, source) in cases {
        assert_eq!(
            validate_source_edit_contract(&batch(target, site, source)),
            Ok(())
        );
    }
}

#[test]
fn contract_rejects_mismatched_or_structurally_injected_declarations() {
    let mismatched = batch(
        SourceNodeRef::structure("types.veac", "Timing"),
        DeclarationSite::EnumDeclaration,
        "enum Timing { Value, }",
    );
    assert_eq!(
        validate_source_edit_contract(&mismatched),
        Err(SourceEditError::IncompatibleDeclarationSite)
    );
    let invalid_name = batch(
        SourceNodeRef::struct_field("types.veac", "Timing", "bad.field"),
        DeclarationSite::StructFieldDeclaration,
        "replacement: time",
    );
    assert!(matches!(
        validate_source_edit_contract(&invalid_name),
        Err(SourceEditError::InvalidNodeId(value)) if value == "bad.field"
    ));
    let wrong_kind = batch(
        SourceNodeRef::structure("types.veac", "Timing"),
        DeclarationSite::StructDeclaration,
        "enum Timing { Value, }",
    );
    assert!(matches!(
        validate_source_edit_contract(&wrong_kind),
        Err(SourceEditError::InvalidDeclaration(_))
    ));
    for source in [
        "",
        "duration: time, other: time",
        "duration: time } enum Injected { Value, } struct Resume { field: time",
        "duration:",
    ] {
        let invalid = batch(
            SourceNodeRef::struct_field("types.veac", "Timing", "duration"),
            DeclarationSite::StructFieldDeclaration,
            source,
        );
        assert!(matches!(
            validate_source_edit_contract(&invalid),
            Err(SourceEditError::InvalidDeclaration(_))
        ));
    }
    let injected_variant = batch(
        SourceNodeRef::enum_variant_field("types.veac", "Placement", "Corner", "x"),
        DeclarationSite::EnumVariantFieldDeclaration,
        "x: length }, Injected { y: length",
    );
    assert!(matches!(
        validate_source_edit_contract(&injected_variant),
        Err(SourceEditError::InvalidDeclaration(_))
    ));
}

#[test]
fn declaration_operation_resolves_to_the_indexed_range() {
    let operation = operation(
        SourceNodeRef::enum_variant("types.veac", "Placement", "Center"),
        DeclarationSite::EnumVariantDeclaration,
        "Middle",
    );
    let resolved =
        resolve_source_edit_text(7, &operation, TextRange { start: 4, end: 10 }).unwrap();
    assert_eq!(resolved.operation_index, 7);
    assert_eq!(resolved.edit.replacement, "Middle");
}

fn batch(target: SourceNodeRef, site: DeclarationSite, source: &str) -> SourceEditBatch {
    let mut value = SourceEditBatch::new(
        veac_ir::OperationId::new("op_nominal_declaration").unwrap(),
        revision('a'),
    );
    value.operations.push(operation(target, site, source));
    value
}

fn operation(target: SourceNodeRef, site: DeclarationSite, source: &str) -> SourceEditOperation {
    SourceEditOperation::SetDeclaration {
        target,
        site,
        declaration: DeclarationSource {
            source: source.into(),
        },
    }
}
