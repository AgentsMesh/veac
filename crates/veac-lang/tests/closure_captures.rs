use veac_lang::program::expression::{
    compile_expression, evaluate, Environment, ExpressionContext, PrimitiveType, TypeEnvironment,
    Value, ValueType,
};

fn integer(source: &str) -> i64 {
    let Value::Integer(value) = evaluate(source, &Environment::new()).unwrap() else {
        panic!("closure expression must produce an int");
    };
    value
}

fn code(source: &str, environment: &Environment) -> &'static str {
    evaluate(source, environment).unwrap_err().code()
}

fn primitive(value: PrimitiveType) -> ValueType {
    ValueType::primitive(value)
}

#[test]
fn captures_are_definition_time_snapshots_across_nested_shadowing() {
    let source = "{ let offset = 1; \
        let add = fn(value: int) -> int effect pure { value + offset }; \
        { let offset = 100; add(2) } }";
    assert_eq!(integer(source), 3);

    let nested = "{ let offset = 1; { let offset = 2; \
        let add = fn(value: int) -> int effect pure { value + offset }; \
        { let offset = 100; add(40) } } }";
    assert_eq!(integer(nested), 42);
}

#[test]
fn returned_nested_closures_retain_each_lexical_parameter_snapshot() {
    let source = "{ let make = fn(first: int) -> fn(int) -> fn(int) -> int effect pure effect pure effect pure { \
        fn(second: int) -> fn(int) -> int effect pure effect pure { \
        fn(third: int) -> int effect pure { first + second + third } } }; \
        make(10)(20)(12) }";
    assert_eq!(integer(source), 42);
}

#[test]
fn capture_metadata_preserves_first_resolved_occurrence_order() {
    let source = "{ let count = 1; let label = \"captured\"; let enabled = true; \
        fn() -> text effect pure { if enabled { label } else { \
        if count > 0 { \"positive\" } else { \"zero\" } } } }";
    let expected = [
        primitive(PrimitiveType::Boolean),
        primitive(PrimitiveType::Text),
        primitive(PrimitiveType::Integer),
    ];

    let Value::Closure(value) = evaluate(source, &Environment::new()).unwrap() else {
        panic!("expression must produce a closure");
    };
    assert_eq!(value.capture_types(), expected.as_slice());

    let compiled =
        compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap();
    let definitions = compiled.core().closure_definitions();
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].capture_types(), expected.as_slice());
}

#[test]
fn closures_cannot_capture_ambient_external_values() {
    let mut environment = Environment::new();
    environment.insert("outside".to_owned(), Value::Integer(40));
    let source = "{ let add = fn(value: int) -> int effect pure { value + outside }; add(2) }";
    assert_eq!(
        code(source, &environment),
        "EXPRESSION_CLOSURE_EXTERNAL_CAPTURE"
    );
}

#[test]
fn closures_cannot_capture_function_typed_locals() {
    let source = "{ let inner = fn(value: int) -> int effect pure { value }; \
        let outer = fn(value: int) -> int effect pure { inner(value) }; outer(1) }";
    assert_eq!(
        code(source, &Environment::new()),
        "EXPRESSION_CLOSURE_FUNCTION_CAPTURE"
    );
}

#[test]
fn closures_cannot_capture_their_own_binding() {
    let source = "{ let recurse: fn(int) -> int effect pure = fn(value: int) -> int effect pure { \
        if value == 0 { 0 } else { recurse(value - 1) } }; recurse(1) }";
    assert_eq!(
        code(source, &Environment::new()),
        "EXPRESSION_CLOSURE_SELF_CAPTURE"
    );
}
