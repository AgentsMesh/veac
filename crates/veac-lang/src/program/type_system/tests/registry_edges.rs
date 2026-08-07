use std::sync::Arc;

use super::super::*;
use crate::program::expression::{PrimitiveType, ValueType};
use crate::program::DomainType;

fn field(index: usize, name: impl Into<Arc<str>>, value_type: ValueType) -> FieldDefinition {
    FieldDefinition::new(FieldIndex::new(index as u16), name, value_type)
}

fn structure(source: &str, name: &str, fields: Vec<FieldDefinition>) -> Arc<TypeDefinition> {
    Arc::new(TypeDefinition::new(
        source,
        name,
        TypeDefinitionKind::Struct(StructDefinition::new(fields)),
    ))
}

fn registry(value: Arc<TypeDefinition>) -> TypeRegistry {
    let mut builder = TypeRegistryBuilder::default();
    builder.insert(Arc::clone(&value)).unwrap();
    builder
        .bind(value.declared_name(), value.type_ref().clone())
        .unwrap();
    builder.finish().unwrap()
}

#[test]
fn builder_idempotence_merge_lookup_and_collision_are_explicit() {
    let first = structure(
        "edge.veac",
        "Edge",
        vec![field(0, "value", PrimitiveType::Text.into())],
    );
    let source = registry(Arc::clone(&first));
    assert!(!source.is_empty());
    assert!(source.retained_bytes() > 0);

    let mut builder = TypeRegistryBuilder::default();
    assert!(builder.resolve("Edge").is_none());
    builder.insert(Arc::clone(&first)).unwrap();
    builder.insert(Arc::clone(&first)).unwrap();
    builder.bind("Edge", first.type_ref().clone()).unwrap();
    assert_eq!(builder.resolve("Edge").unwrap().id(), first.type_ref().id());
    let conflicting = structure(
        "edge.veac",
        "Edge",
        vec![field(0, "changed", PrimitiveType::Text.into())],
    );
    assert_eq!(
        builder.insert(conflicting).unwrap_err().code(),
        "TYPE_ID_COLLISION"
    );

    let mut merged = TypeRegistryBuilder::new();
    merged.merge_definitions(&source).unwrap();
    assert!(merged.resolve("Edge").is_none());
    let mut complete = TypeRegistryBuilder::new();
    complete.merge(&source).unwrap();
    assert_eq!(complete.finish().unwrap().len(), 1);
}

#[test]
fn registry_rejects_unknown_alias_and_duplicate_alias() {
    let missing = TypeRef::new(TypeId::derive("missing.veac", "Missing"), "Missing");
    let mut unknown = TypeRegistryBuilder::new();
    unknown.bind("Missing", missing).unwrap();
    assert_eq!(unknown.finish().unwrap_err().code(), "TYPE_UNKNOWN_ID");

    let value = structure("edge.veac", "Edge", vec![]);
    let mut duplicate = TypeRegistryBuilder::new();
    duplicate.insert(Arc::clone(&value)).unwrap();
    duplicate.bind("Edge", value.type_ref().clone()).unwrap();
    assert_eq!(
        duplicate
            .bind("Edge", value.type_ref().clone())
            .unwrap_err()
            .code(),
        "TYPE_DUPLICATE_NAME"
    );
}

#[test]
fn composite_layout_walks_every_value_type_shape() {
    let mut fields = [
        PrimitiveType::Integer,
        PrimitiveType::Scalar,
        PrimitiveType::Time,
        PrimitiveType::Length,
        PrimitiveType::Percent,
        PrimitiveType::Angle,
        PrimitiveType::Text,
        PrimitiveType::Color,
        PrimitiveType::Boolean,
        PrimitiveType::Identifier,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, kind)| field(index, format!("primitive{index}"), kind.into()))
    .collect::<Vec<_>>();
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let shapes = [
        ValueType::domain(DomainType::Project),
        ValueType::list(integer.clone()).unwrap(),
        ValueType::range(integer.clone()).unwrap(),
        ValueType::map(PrimitiveType::Text.into(), integer.clone()).unwrap(),
        ValueType::map(PrimitiveType::Identifier.into(), integer.clone()).unwrap(),
        ValueType::tuple(vec![integer.clone(), PrimitiveType::Text.into()]).unwrap(),
        ValueType::function(
            vec![integer.clone()],
            PrimitiveType::Boolean.into(),
            crate::program::expression::FunctionEffect::Pure,
        )
        .unwrap(),
    ];
    for (offset, value_type) in shapes.into_iter().enumerate() {
        let index = fields.len();
        fields.push(field(index, format!("shape{offset}"), value_type));
    }
    let value = structure("shapes.veac", "Shapes", fields);
    let id = value.type_ref().id();
    let types = registry(value);
    assert_eq!(types.contains_function(id), Some(true));
    assert_eq!(types.contains_function(TypeId::from_bytes([9; 32])), None);
}

#[test]
fn validation_rejects_invalid_names_members_and_indices() {
    let invalid_name = structure("bad.veac", "bad.name", vec![]);
    assert_eq!(
        TypeRegistryBuilder::new()
            .insert(invalid_name)
            .unwrap_err()
            .code(),
        "TYPE_DECLARED_NAME"
    );
    let duplicate = structure(
        "bad.veac",
        "Duplicate",
        vec![
            field(0, "same", PrimitiveType::Text.into()),
            field(1, "same", PrimitiveType::Text.into()),
        ],
    );
    assert_eq!(insert_error(duplicate), "TYPE_DUPLICATE_MEMBER");
    let sparse = structure(
        "bad.veac",
        "Sparse",
        vec![field(1, "value", PrimitiveType::Text.into())],
    );
    assert_eq!(insert_error(sparse), "TYPE_LAYOUT_INDEX");

    let members = (0..=MAX_TYPE_MEMBERS)
        .map(|index| field(index, format!("field{index}"), PrimitiveType::Text.into()))
        .collect();
    assert_eq!(
        insert_error(structure("bad.veac", "Large", members)),
        "TYPE_MEMBER_LIMIT"
    );
}

#[test]
fn enum_validation_checks_variant_and_payload_layouts() {
    let duplicate = Arc::new(TypeDefinition::new(
        "bad.veac",
        "DuplicateEnum",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![
            EnumVariantDefinition::new(VariantIndex::new(0), "Same", vec![]),
            EnumVariantDefinition::new(VariantIndex::new(1), "Same", vec![]),
        ])),
    ));
    assert_eq!(insert_error(duplicate), "TYPE_DUPLICATE_MEMBER");
    let sparse = Arc::new(TypeDefinition::new(
        "bad.veac",
        "SparseEnum",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(1),
            "Only",
            vec![],
        )])),
    ));
    assert_eq!(insert_error(sparse), "TYPE_LAYOUT_INDEX");
    let payload = Arc::new(TypeDefinition::new(
        "bad.veac",
        "PayloadEnum",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Only",
            vec![field(1, "value", PrimitiveType::Text.into())],
        )])),
    ));
    assert_eq!(insert_error(payload), "TYPE_LAYOUT_INDEX");
}

fn insert_error(value: Arc<TypeDefinition>) -> &'static str {
    TypeRegistryBuilder::new().insert(value).unwrap_err().code()
}
