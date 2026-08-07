use std::sync::Arc;

use super::super::{function_with_signatures, SymbolTarget};
use crate::program::expression::hir::TypedNodeKind;
use crate::program::expression::{
    compile_functions, ExpressionContext, FunctionDefinition, FunctionParameter, PrimitiveType,
    ValueType,
};
use crate::program::{
    FieldDefinition, FieldIndex, MethodDefinition, MethodRegistryBuilder, MethodSignature,
    MethodVisibility, StructDefinition, TypeDefinition, TypeDefinitionKind, TypeRegistryBuilder,
};

fn parse(source: &str) -> crate::program::expression::ast::Expression {
    crate::program::expression::parser::parse(
        crate::program::expression::lexer::lex(source).unwrap(),
    )
    .unwrap()
}

fn context() -> (ExpressionContext, ValueType) {
    let callback = ValueType::function(
        vec![PrimitiveType::Integer.into()],
        integer(),
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "Subject",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "callback",
            callback,
        )])),
    ));
    let receiver = definition.type_ref().clone();
    let mut types = TypeRegistryBuilder::new();
    types.insert(Arc::clone(&definition)).unwrap();
    types.bind("Subject", receiver.clone()).unwrap();
    let types = Arc::new(types.finish().unwrap());
    let signature = MethodSignature::new(
        receiver.clone(),
        "finish",
        vec![FunctionParameter::new("extra", time())],
        time(),
    );
    let method = Arc::new(MethodDefinition::new(
        signature,
        "types.veac",
        MethodVisibility::Private,
    ));
    let mut methods = MethodRegistryBuilder::new();
    methods.insert(method, &types).unwrap();
    let context = ExpressionContext::empty()
        .with_types(types)
        .with_methods(methods.finish());
    (context, ValueType::nominal(receiver))
}

fn lower(
    source: &str,
    receiver: Option<ValueType>,
    context: &ExpressionContext,
) -> Result<
    crate::program::expression::hir::TypedExpression,
    crate::program::expression::ExpressionError,
> {
    function_with_signatures(
        &parse(source),
        &time(),
        &|name| {
            (name == "subject")
                .then(|| receiver.clone())
                .flatten()
                .map(|value| SymbolTarget::Parameter(0, value))
        },
        context,
        &Default::default(),
    )
}

#[test]
fn exact_nominal_method_resolves_to_its_function_id() {
    let (context, receiver) = context();
    let typed = lower("subject.finish(1s)", Some(receiver), &context).unwrap();
    let TypedNodeKind::MethodCall(call) = typed.root.kind else {
        panic!("call must resolve as a nominal method")
    };
    let expected = context
        .methods()
        .definitions()
        .next()
        .unwrap()
        .signature()
        .function_id();
    assert_eq!(call.target, expected);
    assert!(matches!(call.receiver.kind, TypedNodeKind::Parameter(0)));
    assert_eq!(call.arguments.len(), 1);
}

#[test]
fn method_diagnostics_distinguish_lookup_receiver_arity_and_type() {
    let (context, receiver) = context();
    let fixtures = [
        (
            "subject.missing(1s)",
            receiver.clone(),
            "EXPRESSION_UNKNOWN_METHOD",
        ),
        (
            "subject.finish(1s)",
            integer(),
            "EXPRESSION_METHOD_RECEIVER_TYPE",
        ),
        (
            "subject.finish()",
            receiver.clone(),
            "EXPRESSION_CALL_ARITY",
        ),
        (
            "subject.finish(1)",
            receiver,
            "EXPRESSION_CALL_ARGUMENT_TYPE",
        ),
    ];
    for (source, receiver, code) in fixtures {
        assert_eq!(
            lower(source, Some(receiver), &context).unwrap_err().code(),
            code
        );
    }
}

#[test]
fn function_valued_fields_remain_invocable_when_no_method_exists() {
    let (context, receiver) = context();
    let typed = function_with_signatures(
        &parse("subject.callback(41)"),
        &integer(),
        &|name| (name == "subject").then(|| SymbolTarget::Parameter(0, receiver.clone())),
        &context,
        &Default::default(),
    )
    .unwrap();
    assert!(matches!(typed.root.kind, TypedNodeKind::Invoke { .. }));
}

#[test]
fn full_static_name_wins_but_a_value_head_selects_the_method() {
    let (method_context, receiver) = context();
    let static_context = compile_functions(
        &ExpressionContext::empty(),
        &[FunctionDefinition::new(
            "subject.finish",
            Vec::new(),
            time(),
            "{ 2s }",
        )],
    )
    .unwrap()
    .with_types(Arc::new(method_context.types().clone()))
    .with_methods(method_context.methods().clone());
    let static_call = lower("subject.finish()", None, &static_context).unwrap();
    assert!(matches!(static_call.root.kind, TypedNodeKind::Call { .. }));

    let method_call = lower("subject.finish(1s)", Some(receiver), &static_context).unwrap();
    assert!(matches!(
        method_call.root.kind,
        TypedNodeKind::MethodCall(_)
    ));
}

fn integer() -> ValueType {
    PrimitiveType::Integer.into()
}

fn time() -> ValueType {
    PrimitiveType::Time.into()
}
