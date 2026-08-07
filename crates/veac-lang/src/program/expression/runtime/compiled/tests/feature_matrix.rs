use std::sync::Arc;

use crate::program::expression::{
    compile_expression, evaluate, evaluate_compiled, evaluate_in, Environment, ExpressionContext,
    PrimitiveType, TypeEnvironment, Value, ValueType, MAX_TEXT_VALUE_BYTES,
};
use crate::program::{
    EnumDefinition, EnumVariantDefinition, FieldDefinition, FieldIndex, StructDefinition,
    TypeDefinition, TypeDefinitionKind, TypeRegistryBuilder, VariantIndex,
};

fn rendered(source: &str) -> String {
    evaluate(source, &Environment::new()).unwrap().render()
}

#[test]
fn structural_range_and_aggregate_instructions_execute_as_one_verified_program() {
    let source = r#"{
        let pair = (identifier("asset"), [1, 2]);
        let counts = map(#{"b": 2, "a": 1}, fn(entry: (text, int)) -> int effect pure { 1 });
        let selected = filter(counts, fn(value: int) -> bool effect pure { value == 1 });
        let total = fold(selected, 0, fn(total: int, value: int) -> int effect pure { total + value });
        (pair, total, map(3 .. 0 by -1, fn(value: int) -> int effect pure { value * 2 }))
    }"#;
    assert_eq!(
        rendered(source),
        "((identifier(\"asset\"), [1, 2]), 2, [6, 4, 2])"
    );
}

#[test]
fn compiled_input_binding_reports_missing_runtime_type_and_size_contracts() {
    let mut types = TypeEnvironment::new();
    types.insert(
        "input".to_owned(),
        ValueType::primitive(PrimitiveType::Integer),
    );
    let compiled = compile_expression("input + 1", &types, &ExpressionContext::empty()).unwrap();
    assert_eq!(
        evaluate_compiled(&compiled, &Environment::new())
            .unwrap_err()
            .code(),
        "EXPRESSION_UNKNOWN_SYMBOL"
    );
    let wrong = Environment::from([("input".to_owned(), Value::Bool(true))]);
    assert_eq!(
        evaluate_compiled(&compiled, &wrong).unwrap_err().code(),
        "EXPRESSION_RUNTIME_TYPE"
    );
    let exact = Environment::from([("input".to_owned(), Value::Integer(41))]);
    assert_eq!(
        evaluate_compiled(&compiled, &exact).unwrap(),
        Value::Integer(42)
    );

    let text_types = TypeEnvironment::from([(
        "payload".to_owned(),
        ValueType::primitive(PrimitiveType::Text),
    )]);
    let text = compile_expression("payload", &text_types, &ExpressionContext::empty()).unwrap();
    let oversized = Environment::from([(
        "payload".to_owned(),
        Value::Text("x".repeat(MAX_TEXT_VALUE_BYTES + 1).into()),
    )]);
    assert_eq!(
        evaluate_compiled(&text, &oversized).unwrap_err().code(),
        "EXPRESSION_TEXT_LIMIT"
    );
}

#[test]
fn callable_typed_external_input_cannot_fake_its_verified_contract() {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let callback = ValueType::function(
        vec![integer.clone()],
        integer,
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    let types = TypeEnvironment::from([("callback".to_owned(), callback)]);
    let error = compile_expression("callback(1)", &types, &ExpressionContext::empty()).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXTERNAL_FUNCTION_VALUE");
}

#[test]
fn nominal_construction_projection_and_callback_arguments_execute_from_verified_layouts() {
    let context = nominal_context();
    let source = r#"{
        let read = fn(point: SamplePoint) -> int effect pure { point.x };
        let point = SamplePoint { x: 3 };
        (read(point), Choice.Ready { value: 4 })
    }"#;
    assert_eq!(
        evaluate_in(source, &Environment::new(), &context)
            .unwrap()
            .render(),
        "(3, Choice.Ready { value: 4 })"
    );
}

fn nominal_context() -> ExpressionContext {
    let integer = ValueType::primitive(PrimitiveType::Integer);
    let field = || FieldDefinition::new(FieldIndex::new(0), "x", integer.clone());
    let point = Arc::new(TypeDefinition::new(
        "runtime-matrix.veac",
        "SamplePoint",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![field()])),
    ));
    let choice = Arc::new(TypeDefinition::new(
        "runtime-matrix.veac",
        "Choice",
        TypeDefinitionKind::Enum(EnumDefinition::new(vec![EnumVariantDefinition::new(
            VariantIndex::new(0),
            "Ready",
            vec![FieldDefinition::new(FieldIndex::new(0), "value", integer)],
        )])),
    ));
    let mut builder = TypeRegistryBuilder::new();
    for definition in [&point, &choice] {
        builder.insert(Arc::clone(definition)).unwrap();
        builder
            .bind(definition.declared_name(), definition.type_ref().clone())
            .unwrap();
    }
    ExpressionContext::empty().with_types(Arc::new(builder.finish().unwrap()))
}
