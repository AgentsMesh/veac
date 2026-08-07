use std::sync::Arc;

use crate::program::expression::{
    compile_expression, compile_functions, evaluate_compiled, evaluate_in, ExactNumber,
    ExpressionContext, FunctionDefinition, FunctionOrigin, FunctionParameter, PrimitiveType,
    TypeEnvironment, Value, ValueType,
};

fn scalar(name: &str) -> FunctionParameter {
    FunctionParameter::new(name, ValueType::primitive(PrimitiveType::Scalar))
}

fn function(name: &str, parameters: &[&str], body: &str) -> FunctionDefinition {
    FunctionDefinition::new(
        name,
        parameters.iter().map(|name| scalar(name)).collect(),
        ValueType::primitive(PrimitiveType::Scalar),
        format!("{{{body}}}"),
    )
}

#[test]
fn forward_calls_compile_into_an_executable_dag() {
    let definitions = [
        function("score", &["x"], "double(increment(x))"),
        function("double", &["x"], "x * 2.0"),
        function("increment", &["x"], "x + 1.0"),
    ];
    let context = compile_functions(&ExpressionContext::empty(), &definitions).unwrap();
    let value = evaluate_in("score(20.0)", &Default::default(), &context).unwrap();
    assert_eq!(value, Value::Scalar(ExactNumber::integer(42)));
    assert_eq!(context.functions().len(), 3);
    assert_eq!(
        context.functions().lookup("score").unwrap().return_type(),
        &ValueType::primitive(PrimitiveType::Scalar)
    );
}

#[test]
fn imported_maps_are_cheaply_cloned_and_callable() {
    let imported = compile_functions(
        &ExpressionContext::empty(),
        &[function("base", &["x"], "x + 1.0")],
    )
    .unwrap();
    let cloned = imported.clone();
    assert!(Arc::ptr_eq(
        imported.functions().lookup("base").unwrap(),
        cloned.functions().lookup("base").unwrap()
    ));
    let context =
        compile_functions(&imported, &[function("derived", &["x"], "base(x) * 2.0")]).unwrap();
    let value = evaluate_in("derived(2.0)", &Default::default(), &context).unwrap();
    assert_eq!(value.render(), "6.0");
}

#[test]
fn compiled_expressions_are_typed_and_reusable() {
    let mut types = TypeEnvironment::new();
    types.insert(
        "duration".to_owned(),
        ValueType::primitive(PrimitiveType::Time),
    );
    let compiled =
        compile_expression("duration / 2.0", &types, &ExpressionContext::empty()).unwrap();
    assert_eq!(
        compiled.result_type(),
        &ValueType::primitive(PrimitiveType::Time)
    );

    let mut values = super::super::Environment::new();
    values.insert("duration".to_owned(), Value::Time(ExactNumber::integer(8)));
    assert_eq!(
        evaluate_compiled(&compiled, &values).unwrap().render(),
        "4s"
    );
}

#[test]
fn dynamic_execution_can_exceed_the_static_expression_node_limit() {
    let mut definitions = Vec::new();
    definitions.push(function("f12", &["x"], "x + 1.0"));
    for index in (0..12).rev() {
        definitions.push(function(
            &format!("f{index}"),
            &["x"],
            &format!("f{}(x) + f{}(x)", index + 1, index + 1),
        ));
    }
    let context = compile_functions(&ExpressionContext::empty(), &definitions).unwrap();
    let value = evaluate_in("f0(0.0)", &Default::default(), &context).unwrap();
    assert_eq!(value.render(), "4096.0");
}

#[test]
fn runtime_errors_keep_the_innermost_origin_and_authored_call_frames() {
    let helper =
        function("helper", &["x"], "1.0 / x").with_origin(FunctionOrigin::new("math.veac", 40..49));
    let outer = function("outer", &["x"], "helper(x)")
        .with_origin(FunctionOrigin::new("timing.veac", 100..111));
    let context = compile_functions(&ExpressionContext::empty(), &[outer, helper]).unwrap();

    let error = evaluate_in("outer(0.0)", &Default::default(), &context).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_DIVIDE_BY_ZERO");
    assert_eq!(error.function_name(), Some("helper"));
    assert_eq!(error.authored_origin().unwrap().source_id(), "math.veac");
    assert_eq!(error.authored_span(), Some(41..48));
    assert_eq!(error.call_frames().len(), 1);
    let frame = &error.call_frames()[0];
    assert_eq!(frame.function_name(), "outer");
    assert_eq!(frame.authored_span(), Some(101..110));
}

#[test]
fn definitions_without_authored_origins_keep_the_public_api_contract() {
    let context = compile_functions(
        &ExpressionContext::empty(),
        &[function("inverse", &["x"], "1.0 / x")],
    )
    .unwrap();
    let error = evaluate_in("inverse(0.0)", &Default::default(), &context).unwrap_err();
    assert_eq!(error.function_name(), Some("inverse"));
    assert_eq!(error.authored_origin(), None);
    assert_eq!(error.authored_span(), None);
}
