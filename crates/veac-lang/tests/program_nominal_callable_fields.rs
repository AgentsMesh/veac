use std::sync::Arc;

use veac_lang::program::expression::{
    compile_functions, evaluate_in, Environment, ExpressionContext, FunctionDefinition,
    FunctionParameter, PrimitiveType, Value, ValueType,
};
use veac_lang::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeRegistryBuilder, VariantIndex,
};

#[test]
fn constructed_struct_function_field_is_directly_invocable() {
    let (context, _, _) = context();
    let value = evaluate_in(
        "CallbackBox { callback: fn(value: int) -> int effect pure { value + 1 } }.callback(41)",
        &Environment::new(),
        &context,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(42));
}

#[test]
fn helper_chain_resolves_callable_metadata_through_nominal_parameter() {
    let (context, box_type, _) = context();
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let functions = [
        FunctionDefinition::new(
            "invoke_box",
            vec![FunctionParameter::new("value", box_type.clone())],
            integer.clone(),
            "{ value.callback(41) }",
        ),
        FunctionDefinition::new(
            "relay_box",
            vec![FunctionParameter::new("value", box_type)],
            integer,
            "{ invoke_box(value) }",
        ),
    ];
    let context = compile_functions(&context, &functions).unwrap();
    let value = evaluate_in(
        "relay_box(CallbackBox { callback: fn(value: int) -> int effect pure { value + 1 } })",
        &Environment::new(),
        &context,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(42));
}

#[test]
fn enum_match_binds_the_selected_function_payload_contract() {
    let (context, _, choice_type) = context();
    let function = FunctionDefinition::new(
        "invoke_choice",
        vec![
            FunctionParameter::new("choice", choice_type),
            FunctionParameter::new("value", PrimitiveType::Integer.into()),
        ],
        PrimitiveType::Integer.into(),
        r#"{
            match choice {
                CallbackChoice.Increment { callback } => callback(value),
                CallbackChoice.Double { callback } => callback(value),
            }
        }"#,
    );
    let context = compile_functions(&context, &[function]).unwrap();
    for (source, expected) in [
        (
            "invoke_choice(CallbackChoice.Increment { callback: fn(v: int) -> int effect pure { v + 1 } }, 41)",
            42,
        ),
        (
            "invoke_choice(CallbackChoice.Double { callback: fn(v: int) -> int effect pure { v * 2 } }, 21)",
            42,
        ),
    ] {
        assert_eq!(
            evaluate_in(source, &Environment::new(), &context).unwrap(),
            Value::Integer(expected),
        );
    }
}

fn context() -> (ExpressionContext, ValueType, ValueType) {
    let callback = ValueType::function(
        vec![PrimitiveType::Integer.into()],
        PrimitiveType::Integer.into(),
        veac_lang::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let field = || FieldDefinition::new(FieldIndex::new(0), "callback", callback.clone());
    let structure = Arc::new(TypeDefinition::new(
        "types.veac",
        "CallbackBox",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field()])),
    ));
    let choice = Arc::new(TypeDefinition::new(
        "types.veac",
        "CallbackChoice",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![
            EnumVariantDefinition::new(VariantIndex::new(0), "Increment", vec![field()]),
            EnumVariantDefinition::new(VariantIndex::new(1), "Double", vec![field()]),
        ])),
    ));
    let mut registry = TypeRegistryBuilder::new();
    for definition in [&structure, &choice] {
        registry.insert(Arc::clone(definition)).unwrap();
        registry
            .bind(definition.declared_name(), definition.type_ref().clone())
            .unwrap();
    }
    let context = ExpressionContext::empty().with_types(Arc::new(registry.finish().unwrap()));
    (
        context,
        ValueType::nominal(structure.type_ref().clone()),
        ValueType::nominal(choice.type_ref().clone()),
    )
}
