use std::sync::Arc;

use super::*;

#[test]
fn admission_recomputes_identity_digest_and_dense_indices() {
    let mut identity = brand();
    identity.type_ref = TypeRef::new(TypeId::from_bytes([7; 32]), "Brand");
    assert_eq!(insert_error(identity), "TYPE_ID_MISMATCH");

    let mut digest = brand();
    digest.digest = TypeDefinitionDigest::from_bytes([9; 32]);
    assert_eq!(insert_error(digest), "TYPE_DEFINITION_DIGEST");

    let indices = TypeDefinition::new(
        "types/brand.veac",
        "Brand",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field(1, "title")])),
    );
    assert_eq!(insert_error(indices), "TYPE_LAYOUT_INDEX");
}

#[test]
fn registry_rejects_empty_enums_and_unknown_nominal_references() {
    let empty = TypeDefinition::new(
        "types/placement.veac",
        "Placement",
        TypeDefinitionKind::Enum(EnumDefinition::new(Vec::new())),
    );
    assert_eq!(insert_error(empty), "TYPE_ENUM_EMPTY");

    let missing = TypeRef::new(TypeId::derive("types/missing.veac", "Missing"), "Missing");
    let owner = TypeDefinition::new(
        "types/owner.veac",
        "Owner",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "missing",
            ValueType::nominal(missing),
        )])),
    );
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(owner)).unwrap();
    assert_eq!(builder.finish().unwrap_err().code(), "TYPE_UNKNOWN_ID");
}

#[test]
fn registry_rejects_aliases_that_surface_syntax_cannot_name() {
    let definition = Arc::new(brand());
    for name in ["", ".Brand", "brand..Brand", "brand/Brand"] {
        let mut builder = TypeRegistryBuilder::new();
        builder.insert(Arc::clone(&definition)).unwrap();
        assert_eq!(
            builder
                .bind(name, definition.type_ref().clone())
                .unwrap_err()
                .code(),
            "TYPE_INVALID_NAME",
            "{name}"
        );
    }
}

#[test]
fn registry_reserves_domain_type_names_but_allows_qualified_names() {
    let definition = Arc::new(brand());
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&definition)).unwrap();
    assert_eq!(
        builder
            .bind("Project", definition.type_ref().clone())
            .unwrap_err()
            .code(),
        "TYPE_RESERVED_NAME"
    );
    builder
        .bind("brand.Project", definition.type_ref().clone())
        .unwrap();
}

#[test]
fn cycles_are_rejected_through_structural_and_function_nesting() {
    let a_ref = TypeRef::new(TypeId::derive("types.veac", "A"), "A");
    let b_ref = TypeRef::new(TypeId::derive("types.veac", "B"), "B");
    let a = TypeDefinition::new(
        "types.veac",
        "A",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "values",
            ValueType::list(ValueType::nominal(b_ref)).unwrap(),
        )])),
    );
    let callback = ValueType::function(
        vec![ValueType::nominal(a_ref)],
        ValueType::primitive(PrimitiveType::Integer),
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let b = TypeDefinition::new(
        "types.veac",
        "B",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "callback",
            callback,
        )])),
    );
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(a)).unwrap();
    builder.insert(Arc::new(b)).unwrap();
    let error = builder.finish().unwrap_err();
    assert_eq!(error.code(), "TYPE_RECURSIVE_LAYOUT");
    assert!(error.message().contains("A -> B -> A"));
}

#[test]
fn function_presence_and_retained_limits_are_transitive_and_exact() {
    let leaf_ref = TypeRef::new(TypeId::derive("types.veac", "Leaf"), "Leaf");
    let leaf = TypeDefinition::new(
        "types.veac",
        "Leaf",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "callback",
            ValueType::function(
                Vec::new(),
                ValueType::primitive(PrimitiveType::Text),
                crate::program::expression::FunctionEffect::Pure,
            )
            .unwrap(),
        )])),
    );
    let root = TypeDefinition::new(
        "types.veac",
        "Root",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "leaf",
            ValueType::nominal(leaf_ref),
        )])),
    );
    let root_id = root.type_ref().id();
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(leaf)).unwrap();
    builder.insert(Arc::new(root)).unwrap();
    assert_eq!(
        builder.finish().unwrap().contains_function(root_id),
        Some(true)
    );

    let value = Arc::new(brand());
    let exact = value.retained_bytes().unwrap();
    let mut accepted = TypeRegistryBuilder::with_limit(exact);
    accepted.insert(Arc::clone(&value)).unwrap();
    assert_eq!(accepted.finish().unwrap().retained_bytes(), exact);
    let mut rejected = TypeRegistryBuilder::with_limit(exact - 1);
    assert_eq!(
        rejected.insert(value).unwrap_err().code(),
        "TYPE_REGISTRY_LIMIT"
    );
}

fn insert_error(value: TypeDefinition) -> &'static str {
    TypeRegistryBuilder::new()
        .insert(Arc::new(value))
        .unwrap_err()
        .code()
}
