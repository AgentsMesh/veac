use std::sync::Arc;

use super::*;
use crate::program::expression::{PrimitiveType, ValueType};
use crate::program::{
    MethodBody, MethodDefinition, MethodRegistryBuilder, MethodSignature, MethodVisibility,
    StructDefinition, TypeDefinition, TypeDefinitionKind, TypeRegistryBuilder,
};

#[test]
fn authored_method_origin_span_is_part_of_the_semantic_key() {
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "Subject",
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    ));
    let mut types = TypeRegistryBuilder::new();
    types.insert(Arc::clone(&definition)).unwrap();
    types
        .bind("Subject", definition.type_ref().clone())
        .unwrap();
    let types = Arc::new(types.finish().unwrap());
    let signature = MethodSignature::new(
        definition.type_ref().clone(),
        "value",
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
    );
    let mut methods = MethodRegistryBuilder::new();
    methods
        .insert(
            Arc::new(
                MethodDefinition::new(signature, "types.veac", MethodVisibility::Private)
                    .with_body(MethodBody::new(
                        "{ 1 }",
                        crate::program::expression::FunctionOrigin::new("types.veac", 1..6),
                    )),
            ),
            &types,
        )
        .unwrap();
    let first = ExpressionContext::empty()
        .with_types(Arc::clone(&types))
        .with_methods(methods.finish());
    let mut changed_methods = MethodRegistryBuilder::new();
    let signature = MethodSignature::new(
        definition.type_ref().clone(),
        "value",
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
    );
    changed_methods
        .insert(
            Arc::new(
                MethodDefinition::new(signature, "types.veac", MethodVisibility::Private)
                    .with_body(MethodBody::new(
                        "{ 1 }",
                        crate::program::expression::FunctionOrigin::new("types.veac", 2..7),
                    )),
            ),
            &types,
        )
        .unwrap();
    let changed = first.clone().with_methods(changed_methods.finish());
    assert_ne!(
        FunctionQueryKey::new(&first, &[]),
        FunctionQueryKey::new(&changed, &[])
    );
}

#[test]
fn namespace_bindings_and_default_origins_are_part_of_the_semantic_key() {
    let first = ExpressionContext::empty();
    let mut second_map = crate::program::expression::FunctionMap::new();
    second_map.register_namespace("alias");
    let second = first.clone().with_functions(second_map);
    assert_ne!(
        FunctionQueryKey::new(&first, &[]),
        FunctionQueryKey::new(&second, &[])
    );

    let parameter = |span| {
        crate::program::expression::FunctionParameter::new(
            "duration",
            ValueType::primitive(PrimitiveType::Time),
        )
        .with_default(
            "1s",
            Some(crate::program::expression::FunctionOrigin::new(
                "main.veac",
                span,
            )),
        )
    };
    let one = crate::program::expression::FunctionDefinition::new(
        "render",
        vec![parameter(1..3)],
        ValueType::primitive(PrimitiveType::Time),
        "{ 1s }",
    );
    let two = crate::program::expression::FunctionDefinition::new(
        "render",
        vec![parameter(2..4)],
        ValueType::primitive(PrimitiveType::Time),
        "{ 1s }",
    );
    assert_ne!(
        FunctionQueryKey::new(&first, &[one]),
        FunctionQueryKey::new(&first, &[two])
    );
}
