use std::collections::BTreeMap;
use std::sync::Arc;

use crate::program::expression::{
    evaluate, evaluate_in, evaluate_lookup_with_functions, ExpressionContext, PrimitiveType, Value,
    ValueType,
};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeId, TypeRegistry, TypeRegistryBuilder, VariantIndex,
};

#[test]
fn nominal_function_wrapper_is_not_a_trusted_callable_input() {
    let (registry, id) = registry();
    let callback = evaluate(
        "fn() -> int effect pure { 1 }",
        &crate::program::expression::Environment::new(),
    )
    .unwrap();
    let value = Value::structure(&registry, id, vec![callback]).unwrap();
    let values = BTreeMap::from([("box".to_owned(), Arc::new(value))]);
    let context = ExpressionContext::empty().with_types(Arc::new(registry));

    let error = evaluate_lookup_with_functions("box", &values, &context).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXTERNAL_FUNCTION_VALUE");
}

#[test]
fn nominal_function_wrapper_keeps_equality_and_capture_safety_rules() {
    let (registry, _) = registry();
    let context = ExpressionContext::empty().with_types(Arc::new(registry));
    let environment = crate::program::expression::Environment::new();
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
fn nominal_function_wrapper_flows_through_verified_collection_metadata() {
    let (registry, _) = registry();
    let context = ExpressionContext::empty().with_types(Arc::new(registry));
    let source = "map([CallbackBox { callback: fn() -> int effect pure { 42 } }], \
        fn(value: CallbackBox) -> int effect pure { value.callback() })";
    let value = evaluate_in(
        source,
        &crate::program::expression::Environment::new(),
        &context,
    )
    .unwrap();
    assert_eq!(value.render(), "[42]");
}

#[test]
fn nominal_capture_hidden_in_wrapped_callback_uses_its_lexical_registry() {
    let (registry, _) = registry();
    let context = ExpressionContext::empty().with_types(Arc::new(registry));
    let source = r#"{
      let locale = Locale.zh-Hans;
      let callbacks = [CallbackBox {
        callback: fn() -> int effect pure {
          match locale { Locale.zh-Hans => 42, Locale.zh-Hant => 0, }
        },
      }];
      map(callbacks, fn(value: CallbackBox) -> int effect pure { value.callback() })
    }"#;
    let value = evaluate_in(
        source,
        &crate::program::expression::Environment::new(),
        &context,
    )
    .unwrap();
    assert_eq!(value.render(), "[42]");
}

fn registry() -> (TypeRegistry, TypeId) {
    let callback = ValueType::function(
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
        crate::program::expression::FunctionEffect::Pure,
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
    let id = definition.type_ref().id();
    let locale = Arc::new(TypeDefinition::new(
        "types.veac",
        "Locale",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![
            EnumVariantDefinition::new(VariantIndex::new(0), "zh-Hans", vec![]),
            EnumVariantDefinition::new(VariantIndex::new(1), "zh-Hant", vec![]),
        ])),
    ));
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(definition.clone()).unwrap();
    builder.insert(locale.clone()).unwrap();
    builder
        .bind("CallbackBox", definition.type_ref().clone())
        .unwrap();
    builder.bind("Locale", locale.type_ref().clone()).unwrap();
    (builder.finish().unwrap(), id)
}
