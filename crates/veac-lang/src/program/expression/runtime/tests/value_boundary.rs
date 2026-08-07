use std::sync::Arc;

use super::super::{reject_external_function, validate_value_ref};
use crate::program::expression::{
    evaluate, ClosureValue, Environment, MapKeyType, PrimitiveType, Value, ValueType,
    MAX_TEXT_VALUE_BYTES,
};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeRegistry, TypeRegistryBuilder, VariantIndex,
};

#[test]
fn external_function_guard_recurses_through_every_aggregate_shape() {
    let callback = evaluate("fn() -> int effect pure { 1 }", &Environment::new()).unwrap();
    let callback_type = callback.value_type();
    let (registry, structure, enumeration) = registry(callback_type.clone());
    let cases = vec![
        callback.clone(),
        Value::list(callback_type.clone(), vec![callback.clone()]).unwrap(),
        Value::tuple(vec![Value::Integer(0), callback.clone()]).unwrap(),
        Value::structure(&registry, structure, vec![callback.clone()]).unwrap(),
        Value::variant(
            &registry,
            enumeration,
            VariantIndex::new(0),
            vec![callback.clone()],
        )
        .unwrap(),
        Value::map(
            MapKeyType::Text,
            callback_type,
            vec![(Value::Text("callback".into()), callback)],
        )
        .unwrap(),
    ];
    for value in cases {
        let error = reject_external_function(&value, &(4..8)).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_EXTERNAL_FUNCTION_VALUE");
        assert_eq!(error.span(), 4..8);
    }
    reject_external_function(&Value::Integer(1), &(0..1)).unwrap();
}

#[test]
fn value_size_guard_recurses_through_collections_nominals_and_captures() {
    let oversized = Value::Text("x".repeat(MAX_TEXT_VALUE_BYTES + 1).into());
    let text_type = ValueType::primitive(PrimitiveType::Text);
    let (registry, structure, enumeration) = registry(text_type.clone());
    let Value::Closure(template) =
        evaluate("fn() -> int effect pure { 1 }", &Environment::new()).unwrap()
    else {
        unreachable!()
    };
    let captured = Value::Closure(Arc::new(ClosureValue::new(
        Arc::clone(template.definition()),
        vec![oversized.clone()],
        Arc::clone(template.registry()),
        template.provenance().cloned(),
        oversized.retained_bytes(),
    )));
    let cases = vec![
        Value::list(text_type.clone(), vec![oversized.clone()]).unwrap(),
        Value::tuple(vec![Value::Integer(0), oversized.clone()]).unwrap(),
        Value::structure(&registry, structure, vec![oversized.clone()]).unwrap(),
        Value::variant(
            &registry,
            enumeration,
            VariantIndex::new(0),
            vec![oversized.clone()],
        )
        .unwrap(),
        Value::map(
            MapKeyType::Text,
            text_type,
            vec![(Value::Text("key".into()), oversized)],
        )
        .unwrap(),
        captured,
    ];
    for value in cases {
        assert_eq!(
            validate_value_ref(&value, &(12..16)).unwrap_err().code(),
            "EXPRESSION_TEXT_LIMIT"
        );
    }
}

fn registry(
    field_type: ValueType,
) -> (TypeRegistry, crate::program::TypeId, crate::program::TypeId) {
    let field = || FieldDefinition::new(FieldIndex::new(0), "value", field_type.clone());
    let structure = TypeDefinition::new(
        "runtime-boundary.veac",
        "Box",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field()])),
    );
    let enumeration = TypeDefinition::new(
        "runtime-boundary.veac",
        "Choice",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Some",
            vec![field()],
        )])),
    );
    let ids = (structure.type_ref().id(), enumeration.type_ref().id());
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(structure)).unwrap();
    builder.insert(Arc::new(enumeration)).unwrap();
    (builder.finish().unwrap(), ids.0, ids.1)
}
