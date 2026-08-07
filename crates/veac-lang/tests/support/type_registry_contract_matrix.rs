use std::sync::Arc;

use crate::program::expression::{PrimitiveType, ValueType};
use crate::program::{
    FieldDefinition, FieldIndex, StructDefinition, TypeDefinition, TypeDefinitionKind, TypeId,
    TypeRef, TypeRegistryBuilder,
};

fn structure(source: &str, name: &str, field: &str, value_type: ValueType) -> Arc<TypeDefinition> {
    Arc::new(TypeDefinition::new(
        source,
        name,
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            field,
            value_type,
        )])),
    ))
}

#[test]
fn registry_reports_identity_alias_and_unknown_reference_contracts() {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let first = structure("types.veac", "Brand", "value", integer.clone());
    let conflict = structure("types.veac", "Brand", "other", integer);
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(first.clone()).unwrap();
    let collision = builder.insert(conflict).unwrap_err();
    assert_eq!(collision.code(), "TYPE_ID_COLLISION");
    assert!(collision
        .to_string()
        .contains(&first.type_ref().id().to_string()));

    let alias = first.type_ref().with_diagnostic_name("brand.Brand");
    assert_eq!(
        first.type_ref().partial_cmp(&alias),
        Some(std::cmp::Ordering::Equal)
    );
    builder.bind("brand.Brand", alias.clone()).unwrap();
    assert_eq!(builder.resolve("brand.Brand"), Some(&alias));
    assert_eq!(
        builder.bind("brand.Brand", alias).unwrap_err().code(),
        "TYPE_DUPLICATE_NAME"
    );

    let mut unknown = TypeRegistryBuilder::new();
    unknown
        .bind(
            "Missing",
            TypeRef::new(TypeId::from_bytes([7; 32]), "Missing"),
        )
        .unwrap();
    assert_eq!(unknown.finish().unwrap_err().code(), "TYPE_UNKNOWN_ID");
}

#[test]
fn registry_detects_canonical_equal_name_cycles() {
    let left = TypeRef::new(TypeId::derive("left.veac", "Node"), "Node");
    let right = TypeRef::new(TypeId::derive("right.veac", "Node"), "Node");
    let first = structure(
        "left.veac",
        "Node",
        "next",
        ValueType::nominal(right.clone()),
    );
    let second = structure(
        "right.veac",
        "Node",
        "next",
        ValueType::nominal(left.clone()),
    );
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(first).unwrap();
    builder.insert(second).unwrap();
    let error = builder.finish().unwrap_err();
    assert_eq!(error.code(), "TYPE_RECURSIVE_LAYOUT");
    assert!(error.message().contains("Node -> Node -> Node"));
}

#[test]
fn registry_function_detection_walks_nested_structural_types() {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let callback = ValueType::function(
        vec![integer.clone()],
        integer.clone(),
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let tuple = ValueType::tuple(vec![integer.clone(), callback]).unwrap();
    let map = ValueType::map(
        ValueType::primitive(PrimitiveType::Text),
        ValueType::list(tuple).unwrap(),
    )
    .unwrap();
    let callable = structure("types.veac", "Callable", "payload", map);
    let plain = structure("types.veac", "Plain", "value", integer);
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(callable.clone()).unwrap();
    builder.insert(plain.clone()).unwrap();
    let registry = builder.finish().unwrap();
    assert_eq!(
        registry.contains_function(callable.type_ref().id()),
        Some(true)
    );
    assert_eq!(
        registry.contains_function(plain.type_ref().id()),
        Some(false)
    );
    assert_eq!(
        registry.contains_function(TypeId::from_bytes([8; 32])),
        None
    );
}
