use std::sync::Arc;

use veac_lang::program::expression::{
    evaluate_in, Environment, ExpressionContext, PrimitiveType, ValueType,
};
use veac_lang::program::{
    FieldDefinition, FieldIndex, StructDefinition, TypeDefinition, TypeDefinitionKind,
    TypeRegistryBuilder,
};

#[test]
fn nominal_function_wrapper_keeps_equality_and_capture_safety_rules() {
    let context = callback_context();
    let environment = Environment::new();
    let value = "CallbackBox { callback: fn() -> int effect pure { 1 } }";
    let cases = [
        (
            format!("{value} == {value}"),
            "EXPRESSION_FUNCTION_EQUALITY",
        ),
        (
            format!("{{ let value = {value}; fn() -> int effect pure {{ value.callback() }} }}"),
            "EXPRESSION_CLOSURE_FUNCTION_CAPTURE",
        ),
    ];
    for (source, code) in cases {
        assert_eq!(
            evaluate_in(&source, &environment, &context)
                .unwrap_err()
                .code(),
            code,
            "{source}"
        );
    }
}

#[test]
fn nominal_function_wrapper_can_flow_through_a_collection() {
    let context = callback_context();
    let source = "map([CallbackBox { callback: fn() -> int effect pure { 42 } }], \
        fn(value: CallbackBox) -> int effect pure { value.callback() })";
    let value = evaluate_in(source, &Environment::new(), &context).unwrap();
    assert_eq!(value.render(), "[42]");
}

fn callback_context() -> ExpressionContext {
    let callback = ValueType::function(
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
        veac_lang::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let definition = Arc::new(TypeDefinition::new(
        "types.veac",
        "CallbackBox",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "callback",
            callback,
        )])),
    ));
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::clone(&definition)).unwrap();
    builder
        .bind("CallbackBox", definition.type_ref().clone())
        .unwrap();
    ExpressionContext::empty().with_types(Arc::new(builder.finish().unwrap()))
}
