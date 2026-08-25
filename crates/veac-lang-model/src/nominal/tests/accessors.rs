use super::super::*;
use crate::{FunctionEffect, PrimitiveType, ValueType, ValueTypeKind};
use std::fmt::Write;

fn text_field(index: u16, name: &str) -> FieldDefinition {
    FieldDefinition::new(
        FieldIndex::new(index),
        name,
        ValueType::primitive(PrimitiveType::Text),
    )
}

#[test]
fn definitions_and_indices_expose_hits_and_misses() {
    let field = text_field(0, "title");
    assert_eq!(field.index().value(), 0);
    assert_eq!(field.index().index(), 0);
    assert_eq!(field.name(), "title");
    assert_eq!(field.value_type().as_primitive(), Some(PrimitiveType::Text));
    assert_eq!(FieldIndex::from_position(4).unwrap().value(), 4);
    assert_eq!(VariantIndex::from_position(8).unwrap().index(), 8);
    let structure = StructDefinition::new(vec![field.clone()]);
    assert_eq!(structure.fields(), std::slice::from_ref(&field));
    assert_eq!(structure.field("title"), Some(&structure.fields()[0]));
    assert!(structure.field("missing").is_none());
    let variant = EnumVariantDefinition::new(VariantIndex::new(0), "Title", vec![field.clone()]);
    assert_eq!(variant.name(), "Title");
    assert_eq!(variant.field("title"), Some(&variant.fields()[0]));
    assert!(variant.field("missing").is_none());
    let enumeration = EnumDefinition::new(vec![variant.clone()]);
    assert_eq!(enumeration.variants(), &[variant]);
    assert_eq!(
        enumeration.variant("Title"),
        Some(&enumeration.variants()[0])
    );
    assert!(enumeration.variant("Missing").is_none());
}

#[test]
fn registry_accessors_and_error_display_are_total() {
    let empty = TypeRegistry::default();
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.definitions().len(), 0);
    assert_eq!(empty.names().len(), 0);
    assert_eq!(empty.retained_bytes(), 0);
    let missing = TypeId::from_bytes([3; 32]);
    assert!(empty.definition(missing).is_none());
    assert!(empty.definition_handle(missing).is_none());
    assert!(empty.resolve("Missing").is_none());
    assert!(empty.contains_function(missing).is_none());
    let error = TypeRegistryError::new("TEST", "diagnostic");
    let mut rendered = String::new();
    write!(&mut rendered, "{error}").unwrap();
    assert_eq!(error.code(), "TEST");
    assert_eq!(error.message(), "diagnostic");
    assert_eq!(rendered, "diagnostic");
}

#[test]
fn function_and_nominal_value_shapes_are_inspectable() {
    let callable = ValueType::function(
        vec![PrimitiveType::Integer.into()],
        PrimitiveType::Boolean.into(),
        FunctionEffect::Any,
    )
    .unwrap();
    assert!(callable.is_direct_function());
    assert!(callable
        .contains_function_in(&TypeRegistry::default())
        .unwrap());
    let nominal = TypeRef::new(TypeId::derive("a.veac", "A"), "A");
    let value = ValueType::nominal(nominal.clone());
    assert!(matches!(
        value.kind(),
        ValueTypeKind::Nominal(reference) if reference.id() == nominal.id()
    ));
    assert!(!value.is_numeric());
    assert!(!value.is_direct_function());
}
