use std::sync::Arc;

use super::*;
use crate::program::expression::core::verify::test_support::raw;
use crate::program::expression::{CoreNominalDefinition, CoreType, CoreTypeEntry, CoreTypeId};
use crate::program::{
    EnumDefinition, FieldDefinition, FieldIndex, StructDefinition, TypeDefinition,
    TypeDefinitionKind, TypeId, TypeRef,
};

fn structure(name: &str) -> Arc<TypeDefinition> {
    Arc::new(TypeDefinition::new(
        "coverage.veac",
        name,
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    ))
}

#[test]
fn nominal_table_rejects_invalid_spans_and_definitions() {
    let (mut span, _) = raw("1");
    let invalid_span = std::ops::Range { start: 2, end: 1 };
    span.nominal_definitions.push(CoreNominalDefinition::new(
        structure("Marker"),
        invalid_span,
    ));
    assert!(verify(&span, &[], &[])
        .unwrap_err()
        .message()
        .contains("invalid span"));

    let (mut invalid, _) = raw("1");
    let empty = Arc::new(TypeDefinition::new(
        "coverage.veac",
        "Empty",
        TypeDefinitionKind::Enum(EnumDefinition::new(Vec::new())),
    ));
    invalid
        .nominal_definitions
        .push(CoreNominalDefinition::new(empty, 0..1));
    assert!(verify(&invalid, &[], &[])
        .unwrap_err()
        .message()
        .contains("TYPE_ENUM_EMPTY"));
}

#[test]
fn nominal_registry_rejects_unresolved_transitive_layouts() {
    let missing = TypeRef::new(TypeId::derive("coverage.veac", "Missing"), "Missing");
    let owner = Arc::new(TypeDefinition::new(
        "coverage.veac",
        "Owner",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "missing",
            ValueType::nominal(missing),
        )])),
    ));
    let (mut program, _) = raw("1");
    program
        .nominal_definitions
        .push(CoreNominalDefinition::new(owner, 0..1));
    assert!(verify(&program, &[], &[])
        .unwrap_err()
        .message()
        .contains("Core nominal registry is invalid"));
}

#[test]
fn nominal_table_must_exactly_match_all_references() {
    let (mut extra, _) = raw("1");
    extra
        .nominal_definitions
        .push(CoreNominalDefinition::new(structure("Extra"), 0..1));
    assert!(verify(&extra, &[], &[])
        .unwrap_err()
        .message()
        .contains("equal its transitive type references"));

    let missing = TypeRef::new(TypeId::derive("coverage.veac", "Missing"), "Missing");
    let (mut unknown, _) = raw("1");
    unknown.types.entries.push(CoreTypeEntry {
        id: CoreTypeId::new(1),
        kind: CoreType::Value(ValueType::nominal(missing)),
    });
    assert!(verify(&unknown, &[], &[])
        .unwrap_err()
        .message()
        .contains("unknown TypeId"));
}
