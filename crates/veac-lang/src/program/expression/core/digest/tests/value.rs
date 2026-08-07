use std::collections::BTreeSet;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use super::super::{value, value_type};
use crate::program::expression::{ExactNumber, MapKeyType, PrimitiveType, Value, ValueType};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeRegistry, TypeRegistryBuilder, VariantIndex,
};

fn encoded(value: &Value) -> [u8; 32] {
    let mut digest = Sha256::new();
    value::encode(&mut digest, value);
    digest.finalize().into()
}

#[test]
fn value_digest_covers_every_core_literal_shape() {
    let (registry, structure, enumeration) = nominal_registry();
    let values = vec![
        Value::Integer(-7),
        Value::Scalar(ExactNumber::new(3, 2).unwrap()),
        Value::Time(ExactNumber::integer(2)),
        Value::Length(ExactNumber::integer(3)),
        Value::Percent(ExactNumber::integer(4)),
        Value::Angle(ExactNumber::integer(5)),
        Value::Text(Arc::from("text")),
        Value::Color(Arc::from("#123456")),
        Value::Bool(true),
        Value::Identifier(Arc::from("key")),
        Value::range(1, 7, 2).unwrap(),
        Value::list(int(), vec![Value::Integer(1), Value::Integer(2)]).unwrap(),
        Value::map(
            MapKeyType::Text,
            int(),
            vec![(Value::Text(Arc::from("a")), Value::Integer(1))],
        )
        .unwrap(),
        Value::tuple(vec![Value::Integer(1), Value::Bool(false)]).unwrap(),
        Value::structure(&registry, structure, vec![Value::Integer(8)]).unwrap(),
        Value::variant(
            &registry,
            enumeration,
            VariantIndex::new(0),
            vec![Value::Text(Arc::from("ready"))],
        )
        .unwrap(),
    ];
    let outputs = values.iter().map(encoded).collect::<BTreeSet<_>>();
    assert_eq!(outputs.len(), values.len());
}

#[test]
fn value_type_digest_covers_closed_type_shapes_and_map_keys() {
    let nominal = TypeDefinition::new(
        "types.veac",
        "Marker",
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    );
    let types = vec![
        int(),
        ValueType::domain(crate::program::DomainType::Canvas),
        ValueType::nominal(nominal.type_ref().clone()),
        ValueType::list(int()).unwrap(),
        ValueType::map(ValueType::from(PrimitiveType::Text), int()).unwrap(),
        ValueType::map(ValueType::from(PrimitiveType::Identifier), int()).unwrap(),
        ValueType::tuple(vec![int(), PrimitiveType::Boolean.into()]).unwrap(),
        ValueType::function(
            vec![int()],
            PrimitiveType::Text.into(),
            crate::program::expression::FunctionEffect::Pure,
        )
        .unwrap(),
        ValueType::range(int()).unwrap(),
    ];
    let mut outputs = BTreeSet::new();
    for kind in types {
        let mut digest = Sha256::new();
        value_type::encode(&mut digest, &kind);
        outputs.insert(<[u8; 32]>::from(digest.finalize()));
    }
    assert_eq!(outputs.len(), 9);
    assert_eq!(
        MapKeyType::Identifier.primitive(),
        PrimitiveType::Identifier
    );
    let mut digest = Sha256::new();
    value::encode(&mut digest, &Value::Identifier(Arc::from("id")));
}

fn nominal_registry() -> (TypeRegistry, crate::program::TypeId, crate::program::TypeId) {
    let structure = TypeDefinition::new(
        "types.veac",
        "Box",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "value",
            int(),
        )])),
    );
    let enumeration = TypeDefinition::new(
        "types.veac",
        "State",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Ready",
            vec![FieldDefinition::new(
                FieldIndex::new(0),
                "label",
                PrimitiveType::Text.into(),
            )],
        )])),
    );
    let ids = (structure.type_ref().id(), enumeration.type_ref().id());
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(structure)).unwrap();
    builder.insert(Arc::new(enumeration)).unwrap();
    (builder.finish().unwrap(), ids.0, ids.1)
}

fn int() -> ValueType {
    PrimitiveType::Integer.into()
}
