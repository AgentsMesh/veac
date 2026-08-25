use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::*;
use crate::{PrimitiveType, ValueType};

fn field(index: u16, name: &str) -> FieldDefinition {
    FieldDefinition::new(
        FieldIndex::new(index),
        name,
        ValueType::primitive(PrimitiveType::Text),
    )
}

fn brand() -> TypeDefinition {
    TypeDefinition::new(
        "types/brand.veac",
        "Brand",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field(0, "title")])),
    )
}

#[test]
fn type_identity_depends_only_on_source_and_declared_name() {
    let first = brand();
    let edited = TypeDefinition::new(
        "types/brand.veac",
        "Brand",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field(0, "label")])),
    );
    assert_eq!(first.type_ref(), edited.type_ref());
    assert_ne!(first.digest(), edited.digest());
    assert_ne!(
        first.type_ref().id(),
        TypeId::derive("types/other.veac", "Brand")
    );
}

#[test]
fn identity_and_definition_digest_have_fixed_v1_vectors() {
    let value = brand();
    assert_eq!(
        value.type_ref().id().to_string(),
        "e7b6f80665760b7b13630c59a73f7b9b67da10eaffaf054d8ca407e7f6787a1e"
    );
    assert_eq!(
        value.digest().to_string(),
        "a873bbe57064adb95f6626984ab10653040369c3c5fb4e8d06ee9536515007d3"
    );
}

#[test]
fn diagnostic_aliases_share_identity_equality_order_and_hash() {
    let original = brand().type_ref().clone();
    let alias = original.with_diagnostic_name("brand.Brand");
    assert_eq!(original, alias);
    assert_eq!(hash(&original), hash(&alias));
    assert_eq!(original.cmp(&alias), std::cmp::Ordering::Equal);
    assert_eq!(alias.diagnostic_name(), "brand.Brand");
}

#[test]
fn registry_admits_verified_layout_and_resolves_aliases() {
    let definition = Arc::new(brand());
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&definition)).unwrap();
    builder
        .bind(
            "brand.Brand",
            definition.type_ref().with_diagnostic_name("brand.Brand"),
        )
        .unwrap();
    let registry = builder.finish().unwrap();
    let resolved = registry.resolve("brand.Brand").unwrap();
    assert_eq!(resolved.id(), definition.type_ref().id());
    assert_eq!(
        registry.definition(resolved.id()).unwrap(),
        definition.as_ref()
    );
}

fn hash(value: &TypeRef) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[path = "tests/accessors.rs"]
mod accessors;
#[path = "tests/registry.rs"]
mod registry;
#[path = "tests/registry_edges.rs"]
mod registry_edges;
#[path = "tests/registry_limits.rs"]
mod registry_limits;
