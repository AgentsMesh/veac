use std::sync::Arc;

use super::super::*;
use crate::program::expression::{
    compile_functions, ExpressionContext, FunctionOrigin, PrimitiveType, ValueType,
};
use crate::program::{
    MethodBody, MethodDefinition, MethodRegistryBuilder, MethodSignature, MethodVisibility,
    StructDefinition, TypeDefinition, TypeDefinitionKind, TypeRegistryBuilder,
};

#[test]
fn compiled_method_payloads_are_charged_atomically() {
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
        "finish",
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
    );
    let id = signature.function_id();
    let method =
        MethodDefinition::new(signature, "types.veac", MethodVisibility::Private).with_body(
            MethodBody::new("{ 1 }", FunctionOrigin::new("types.veac", 50..55)),
        );
    let mut methods = MethodRegistryBuilder::new();
    methods.insert(Arc::new(method), &types).unwrap();
    let methods = methods.finish();
    let context = ExpressionContext::empty()
        .with_types(types)
        .with_methods(methods.clone());
    let compiled = compile_functions(&context, &[]).unwrap();
    let payload = compiled
        .functions()
        .lookup_by_id(id)
        .unwrap()
        .retained_bytes()
        .unwrap();
    let file = crate::program::parser::parse(
        "types.veac",
        "module { struct Subject {} impl Subject @subject { fn finish(self) -> int { 1 } } }",
    )
    .unwrap();

    let mut exact = Budget {
        bytes: 0,
        limit: payload,
    };
    exact
        .functions(&file, compiled.functions(), &methods)
        .unwrap();
    assert_eq!(exact.bytes, payload);

    let mut short = Budget {
        bytes: 0,
        limit: payload - 1,
    };
    assert!(short
        .functions(&file, compiled.functions(), &methods)
        .is_err());
    assert_eq!(short.bytes, 0);
}
