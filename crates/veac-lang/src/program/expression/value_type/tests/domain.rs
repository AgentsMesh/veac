use std::sync::Arc;

use super::super::*;
use crate::program::{
    DomainType, EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex,
    StructDefinition, TypeDefinition, TypeDefinitionKind, TypeId, TypeRef, TypeRegistry,
    TypeRegistryBuilder, VariantIndex,
};

fn registry() -> (TypeRegistry, TypeRef, TypeRef) {
    let scene = TypeDefinition::new(
        "domain-types.veac",
        "Scene",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "projects",
            ValueType::list(ValueType::domain(DomainType::Project)).unwrap(),
        )])),
    );
    let scene_ref = scene.type_ref().clone();
    let choice = TypeDefinition::new(
        "domain-types.veac",
        "Choice",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Scene",
            vec![FieldDefinition::new(
                FieldIndex::new(0),
                "value",
                ValueType::nominal(scene_ref.clone()),
            )],
        )])),
    );
    let choice_ref = choice.type_ref().clone();
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(scene)).unwrap();
    builder.insert(Arc::new(choice)).unwrap();
    (builder.finish().unwrap(), scene_ref, choice_ref)
}

#[test]
fn domain_types_are_closed_opaque_leaf_types() {
    let value = ValueType::domain(DomainType::Project);
    assert_eq!(value.as_domain(), Some(DomainType::Project));
    assert_eq!(value.as_primitive(), None);
    assert_eq!(value.depth(), 1);
    assert_eq!(value.to_string(), "Project");
    assert!(matches!(
        value.kind(),
        ValueTypeKind::Domain(DomainType::Project)
    ));
    assert!(MapKeyType::try_from(&value).is_err());

    let registry = TypeRegistry::default();
    assert_eq!(value.contains_domain_in(&registry), Some(true));
    assert_eq!(value.supports_equality_in(&registry), Some(false));
    assert_eq!(value.is_public_input_in(&registry), Some(false));
}

#[test]
fn domain_capabilities_are_transitive_through_all_type_shapes() {
    let (registry, scene, choice) = registry();
    for value in [ValueType::nominal(scene), ValueType::nominal(choice)] {
        assert_eq!(value.contains_domain_in(&registry), Some(true));
        assert_eq!(value.supports_equality_in(&registry), Some(false));
        assert_eq!(value.is_public_input_in(&registry), Some(false));
    }

    let domain = ValueType::domain(DomainType::Canvas);
    let list = ValueType::list(domain.clone()).unwrap();
    let map = ValueType::map(ValueType::primitive(PrimitiveType::Text), domain.clone()).unwrap();
    let tuple = ValueType::tuple(vec![list, map]).unwrap();
    let function = ValueType::function(vec![tuple], domain, FunctionEffect::Pure).unwrap();
    assert_eq!(function.contains_domain_in(&registry), Some(true));
    assert_eq!(function.supports_equality_in(&registry), Some(false));
}

#[test]
fn ordinary_types_remain_comparable_and_public_input_safe() {
    let registry = TypeRegistry::default();
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let range = ValueType::range(integer.clone()).unwrap();
    let tuple = ValueType::tuple(vec![range, integer.clone()]).unwrap();
    assert_eq!(tuple.contains_domain_in(&registry), Some(false));
    assert_eq!(tuple.supports_equality_in(&registry), Some(true));
    assert_eq!(tuple.is_public_input_in(&registry), Some(true));

    let callable =
        ValueType::function(vec![integer.clone()], integer, FunctionEffect::Pure).unwrap();
    assert_eq!(callable.contains_domain_in(&registry), Some(false));
    assert_eq!(callable.supports_equality_in(&registry), Some(false));
    assert_eq!(callable.is_public_input_in(&registry), Some(true));
}

#[test]
fn unknown_nominal_types_do_not_gain_domain_capabilities() {
    let missing = TypeRef::new(TypeId::derive("missing.veac", "Missing"), "Missing");
    let value = ValueType::nominal(missing);
    let registry = TypeRegistry::default();
    assert_eq!(value.contains_domain_in(&registry), None);
    assert_eq!(value.supports_equality_in(&registry), None);
    assert_eq!(value.is_public_input_in(&registry), None);
}

#[test]
fn domain_kinds_change_nominal_layout_identity() {
    let project = TypeDefinition::new(
        "digest.veac",
        "Box",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "value",
            ValueType::domain(DomainType::Project),
        )])),
    );
    let sequence = TypeDefinition::new(
        "digest.veac",
        "Box",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "value",
            ValueType::domain(DomainType::Sequence),
        )])),
    );
    assert_ne!(project.digest(), sequence.digest());
}
