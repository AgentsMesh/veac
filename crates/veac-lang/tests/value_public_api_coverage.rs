use std::sync::Arc;

use veac_lang::program::expression::{
    ExactNumber, MapKeyType, PrimitiveType, Value, ValueType, ValueTypeKind,
};
use veac_lang::program::{
    DomainType, EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex,
    StructDefinition, TypeDefinition, TypeDefinitionKind, TypeRegistry, TypeRegistryBuilder,
    VariantIndex,
};

fn int() -> ValueType {
    PrimitiveType::Integer.into()
}

#[test]
fn exact_and_structural_values_expose_canonical_public_state() {
    let exact = ExactNumber::new(-6, -4).unwrap();
    assert_eq!((exact.numerator(), exact.denominator()), (3, 2));
    assert!(!exact.is_zero());
    assert!(ExactNumber::new(1, 0).is_none());

    let range = Value::range(1, 8, 2).unwrap();
    let Value::Range(range_value) = &range else {
        unreachable!()
    };
    assert_eq!(
        (
            range_value.start(),
            range_value.end(),
            range_value.step(),
            range_value.count()
        ),
        (1, 8, 2, 4)
    );
    assert_eq!(range_value.render(), "1 .. 8 by 2");

    let list = Value::list(int(), vec![Value::Integer(1), Value::Integer(2)]).unwrap();
    let Value::List(list_value) = &list else {
        unreachable!()
    };
    assert_eq!(list_value.values().len(), 2);
    assert_eq!(list_value.value_type(), &list.value_type());

    let map = Value::map(
        MapKeyType::Identifier,
        int(),
        vec![(Value::Identifier("key".into()), Value::Integer(3))],
    )
    .unwrap();
    let Value::Map(map_value) = &map else {
        unreachable!()
    };
    assert_eq!(map_value.entries().len(), 1);
    assert_eq!(
        map_value.entries()[0].key(),
        &Value::Identifier("key".into())
    );
    assert_eq!(map_value.entries()[0].value(), &Value::Integer(3));
    assert_eq!(map_value.value_type(), &map.value_type());

    let tuple = Value::tuple(vec![list.clone(), map.clone()]).unwrap();
    let Value::Tuple(tuple_value) = &tuple else {
        unreachable!()
    };
    assert_eq!(tuple_value.values(), &[list, map]);
    assert_eq!(tuple_value.value_type(), &tuple.value_type());
    assert!(tuple.render().contains("identifier(\"key\")"));
}

#[test]
fn construction_failures_publish_stable_codes_messages_and_display() {
    let mismatch = Value::list(int(), vec![Value::Bool(true)]).unwrap_err();
    assert_eq!(mismatch.code(), "VALUE_LIST_ELEMENT_TYPE");
    assert_eq!(mismatch.message(), mismatch.to_string());

    let duplicate = Value::map(
        MapKeyType::Text,
        int(),
        vec![
            (Value::Text("same".into()), Value::Integer(1)),
            (Value::Text("same".into()), Value::Integer(2)),
        ],
    )
    .unwrap_err();
    assert_eq!(duplicate.code(), "VALUE_MAP_DUPLICATE_KEY");
    assert!(duplicate.to_string().contains("duplicate"));
    assert_eq!(
        Value::range(0, 1, 0).unwrap_err().code(),
        "VALUE_RANGE_STEP"
    );
}

#[test]
fn nominal_values_keep_verified_definition_identity_and_fields() {
    let (registry, card_id, state_id) = nominal_registry();
    let card = Value::structure(&registry, card_id, vec![Value::Text("标题".into())]).unwrap();
    let Value::Struct(card_value) = &card else {
        unreachable!()
    };
    assert_eq!(card_value.type_id(), card_id);
    assert_eq!(
        card_value.definition_digest(),
        registry.definition(card_id).unwrap().digest()
    );
    assert_eq!(card_value.fields(), &[Value::Text("标题".into())]);

    let state = Value::variant(
        &registry,
        state_id,
        VariantIndex::new(0),
        vec![Value::Integer(9)],
    )
    .unwrap();
    let Value::Enum(state_value) = &state else {
        unreachable!()
    };
    assert_eq!(state_value.type_id(), state_id);
    assert_eq!(state_value.variant(), VariantIndex::new(0));
    assert_eq!(state_value.fields(), &[Value::Integer(9)]);
    assert_eq!(
        state_value.definition_digest(),
        registry.definition(state_id).unwrap().digest()
    );
}

#[test]
fn value_type_shapes_parse_render_and_report_capabilities() {
    let registry = TypeRegistry::default();
    let values = [
        ValueType::parse("int").unwrap(),
        ValueType::parse("list<map<text, (int, bool)>>").unwrap(),
        ValueType::parse("range<int>").unwrap(),
        ValueType::parse("fn(int, text) -> bool effect pure").unwrap(),
        ValueType::domain(DomainType::Canvas),
    ];
    assert!(matches!(
        values[0].kind(),
        ValueTypeKind::Primitive(PrimitiveType::Integer)
    ));
    assert_eq!(values[0].as_primitive(), Some(PrimitiveType::Integer));
    assert!(values[0].is_numeric());
    assert_eq!(values[4].as_domain(), Some(DomainType::Canvas));
    assert_eq!(values[1].depth(), 4);
    assert_eq!(values[0].supports_equality_in(&registry), Some(true));
    assert_eq!(values[3].supports_equality_in(&registry), Some(false));
    assert_eq!(values[4].contains_domain_in(&registry), Some(true));
    assert_eq!(values[4].is_public_input_in(&registry), Some(false));
    assert_eq!(MapKeyType::Text.primitive(), PrimitiveType::Text);
    assert_eq!(
        MapKeyType::Identifier.primitive(),
        PrimitiveType::Identifier
    );

    let error = ValueType::parse("map<int, text>").unwrap_err();
    assert_eq!(error.code(), "VALUE_TYPE_MAP_KEY");
    assert!(error.message().contains("map key"));
    assert!(!error.span().is_empty());
}

fn nominal_registry() -> (
    TypeRegistry,
    veac_lang::program::TypeId,
    veac_lang::program::TypeId,
) {
    let card = TypeDefinition::new(
        "types.veac",
        "Card",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "title",
            PrimitiveType::Text.into(),
        )])),
    );
    let state = TypeDefinition::new(
        "types.veac",
        "State",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Ready",
            vec![FieldDefinition::new(FieldIndex::new(0), "count", int())],
        )])),
    );
    let ids = (card.type_ref().id(), state.type_ref().id());
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(card)).unwrap();
    builder.insert(Arc::new(state)).unwrap();
    (builder.finish().unwrap(), ids.0, ids.1)
}
