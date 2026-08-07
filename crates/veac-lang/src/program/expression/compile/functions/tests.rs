use std::sync::Arc;

use super::super::super::{
    compile_expression, compile_functions, CoreCallTarget, CoreInstructionKind, ExpressionContext,
    FunctionDefinition, FunctionOrigin, FunctionParameter, PrimitiveType, TypeEnvironment,
    ValueType,
};
use crate::program::{
    FieldDefinition, FieldIndex, MethodBody, MethodDefinition, MethodRegistryBuilder,
    MethodSignature, MethodVisibility, StructDefinition, TypeDefinition, TypeDefinitionKind,
    TypeId, TypeRef, TypeRegistryBuilder,
};

#[test]
fn methods_are_hidden_and_receiver_precedes_explicit_arguments_once() {
    let (context, nominal, method_id) = context();
    let make = FunctionDefinition::new(
        "make",
        vec![FunctionParameter::new(
            "seed",
            ValueType::primitive(PrimitiveType::Integer),
        )],
        nominal.clone(),
        "{ Subject {} }",
    );
    let render =
        FunctionDefinition::new("render", Vec::new(), nominal, "{ make(1).choose(make(2)) }");
    let compiled = compile_functions(&context, &[make, render]).unwrap();
    assert!(compiled.functions().lookup("Subject.choose").is_none());
    assert!(compiled.functions().registry().get(method_id).is_some());

    let make_id = compiled.functions().lookup("make").unwrap().id();
    let render = compiled.functions().lookup("render").unwrap();
    let calls = render
        .body()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .filter_map(|instruction| match instruction.kind() {
            CoreInstructionKind::Call {
                target: CoreCallTarget::User(id),
                ..
            } => Some(*id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(calls, [make_id, make_id, method_id]);
}

#[test]
fn unused_nominal_parameters_embed_definitions_without_unused_core_types() {
    let (context, nominal, _) = context();
    let definition = FunctionDefinition::new(
        "ignore",
        vec![FunctionParameter::new("value", nominal)],
        ValueType::primitive(PrimitiveType::Integer),
        "{ 1 }",
    );
    let compiled = compile_functions(&context, &[definition]).unwrap();
    let body = compiled.functions().lookup("ignore").unwrap().body();
    assert_eq!(body.nominal_definitions().len(), 1);
    assert!(body
        .types()
        .entries()
        .iter()
        .all(|entry| entry.kind().value_type()
            != Some(&ValueType::nominal(
                context.types().resolve("Subject").unwrap().clone()
            ))));
}

#[test]
fn unknown_nominal_signature_is_rejected_before_core_collection() {
    let missing = TypeRef::new(TypeId::derive("missing.veac", "Missing"), "Missing");
    let definition = FunctionDefinition::new(
        "ignore",
        vec![FunctionParameter::new("value", ValueType::nominal(missing))],
        ValueType::primitive(PrimitiveType::Integer),
        "{ 1 }",
    );
    let error = compile_functions(&ExpressionContext::empty(), &[definition]).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_UNKNOWN_TYPE");
}

#[test]
fn stale_same_id_layout_is_rejected_at_context_admission() {
    let original = registry_with_fields(vec![
        field(0, "duration", PrimitiveType::Time),
        field(1, "seed", PrimitiveType::Integer),
    ]);
    let nominal = ValueType::nominal(original.resolve("Subject").unwrap().clone());
    let compiled = compile_functions(
        &ExpressionContext::empty().with_types(Arc::clone(&original)),
        &[FunctionDefinition::new(
            "duration",
            vec![FunctionParameter::new("value", nominal)],
            ValueType::primitive(PrimitiveType::Time),
            "{ value.duration }",
        )],
    )
    .unwrap();
    let reordered = registry_with_fields(vec![
        field(0, "seed", PrimitiveType::Integer),
        field(1, "duration", PrimitiveType::Time),
    ]);
    let stale = compiled.with_types(reordered);
    let error = compile_expression("1", &TypeEnvironment::new(), &stale).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_NOMINAL_ABI");
}

fn context() -> (
    ExpressionContext,
    ValueType,
    crate::program::expression::FunctionId,
) {
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "Subject",
        TypeDefinitionKind::Struct(StructDefinition::new(Vec::new())),
    ));
    let receiver = definition.type_ref().clone();
    let nominal = ValueType::nominal(receiver.clone());
    let mut types = TypeRegistryBuilder::new();
    types.insert(Arc::clone(&definition)).unwrap();
    types.bind("Subject", receiver.clone()).unwrap();
    let types = Arc::new(types.finish().unwrap());

    let signature = MethodSignature::new(
        receiver,
        "choose",
        vec![FunctionParameter::new("other", nominal.clone())],
        nominal.clone(),
    );
    let method_id = signature.function_id();
    let method =
        MethodDefinition::new(signature, "types.veac", MethodVisibility::Private).with_body(
            MethodBody::new("{ other }", FunctionOrigin::new("types.veac", 0..9)),
        );
    let mut methods = MethodRegistryBuilder::new();
    methods.insert(Arc::new(method), &types).unwrap();
    let context = ExpressionContext::empty()
        .with_types(types)
        .with_methods(methods.finish());
    (context, nominal, method_id)
}

fn registry_with_fields(fields: Vec<FieldDefinition>) -> Arc<crate::program::TypeRegistry> {
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "Subject",
        TypeDefinitionKind::Struct(StructDefinition::new(fields)),
    ));
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&definition)).unwrap();
    builder
        .bind("Subject", definition.type_ref().clone())
        .unwrap();
    Arc::new(builder.finish().unwrap())
}

fn field(index: u16, name: &str, value: PrimitiveType) -> FieldDefinition {
    FieldDefinition::new(FieldIndex::new(index), name, ValueType::primitive(value))
}
