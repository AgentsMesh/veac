use veac_lang::program::expression::{
    compile_expression, compile_functions, evaluate, evaluate_in, Effect, Environment,
    ExpressionContext, FunctionDefinition, FunctionParameter, PrimitiveType, TypeEnvironment,
    Value, ValueType,
};

fn integer(source: &str) -> i64 {
    let Value::Integer(value) = evaluate(source, &Environment::new()).unwrap() else {
        panic!("expression must produce an int");
    };
    value
}

fn error(source: &str) -> veac_lang::program::expression::ExpressionError {
    evaluate(source, &Environment::new()).unwrap_err()
}

#[test]
fn mutable_locals_initialize_assign_and_read_in_source_order() {
    assert_eq!(
        integer("{ var total: int = 1; set total = total + 2; total }"),
        3
    );
    assert_eq!(
        integer(
            "{ var total = 0; let selected = if true { set total = 7; total } \
             else { set total = 9; total }; total + selected }"
        ),
        14
    );
}

#[test]
fn mutable_initializer_preserves_core_type_first_use_order() {
    assert_eq!(
        integer("{ var selected = if true { 7 } else { 9 }; selected }"),
        7
    );
}

#[test]
fn mutable_locals_are_lexical_and_can_shadow_outer_bindings() {
    assert_eq!(
        integer("{ var value = 1; let inner = { var value = 8; value }; value + inner }"),
        9
    );
    assert_eq!(
        integer("{ var value = 1; let inner = { set value = 4; value }; value + inner }"),
        8
    );
}

#[test]
fn assignment_requires_a_declared_mutable_target_and_exact_type() {
    for (source, code) in [
        ("{ set missing = 1; 0 }", "EXPRESSION_MUTABLE_TARGET"),
        (
            "{ let fixed = 1; set fixed = 2; fixed }",
            "EXPRESSION_MUTABLE_TARGET",
        ),
        (
            "{ var value: int = 1; set value = true; value }",
            "EXPRESSION_TYPE",
        ),
    ] {
        assert_eq!(error(source).code(), code);
    }
}

#[test]
fn mutable_locals_cannot_be_captured_or_hold_function_values() {
    let capture = error("{ var total = 1; let read = fn() -> int effect pure { total }; read() }");
    assert_eq!(capture.code(), "EXPRESSION_MUTABLE_CAPTURE");

    let function = error(
        "{ var callback: fn(int) -> int effect pure = \
         fn(value: int) -> int effect pure { value }; callback(1) }",
    );
    assert_eq!(function.code(), "EXPRESSION_MUTABLE_FUNCTION");
}

#[test]
fn mutable_instructions_have_local_mutation_evidence_and_dense_slots() {
    let source = "{ var total = 1; set total = total + 1; total }";
    let compiled =
        compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap();
    assert_eq!(compiled.core().local_slots().len(), 1);
    let slot = &compiled.core().local_slots()[0];
    assert_eq!(slot.id().value(), 0);
    assert_eq!(
        compiled.core().value_type(slot.type_id()),
        Some(&ValueType::primitive(PrimitiveType::Integer))
    );
    assert!(slot.metadata().contains_local_mutation());
    assert_eq!(&source[slot.span()], "1");
    let local = compiled
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .filter(|instruction| instruction.metadata().contains_local_mutation())
        .collect::<Vec<_>>();
    assert!(local.len() >= 3);
    assert!(local
        .iter()
        .all(|instruction| instruction.metadata().effect() == Effect::LocalMutation));
}

#[test]
fn function_calls_preserve_inferred_local_mutation_evidence() {
    let function = FunctionDefinition::new(
        "increment",
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
        "{ var total = 1; set total = total + 1; total }",
    );
    let context = compile_functions(&ExpressionContext::empty(), &[function]).unwrap();
    let compiled = compile_expression("increment()", &TypeEnvironment::new(), &context).unwrap();
    let call = compiled.core().blocks()[0].instructions().last().unwrap();
    assert_eq!(call.metadata().effect(), Effect::LocalMutation);
    assert!(call.metadata().contains_local_mutation());
}

#[test]
fn every_function_and_closure_invocation_has_an_isolated_local_activation() {
    let function = FunctionDefinition::new(
        "advance",
        vec![FunctionParameter::new(
            "start",
            ValueType::primitive(PrimitiveType::Integer),
        )],
        ValueType::primitive(PrimitiveType::Integer),
        "{ var current = start; set current = current + 1; current }",
    );
    let context = compile_functions(&ExpressionContext::empty(), &[function]).unwrap();
    let function_calls = evaluate_in(
        "advance(1) * 100 + advance(40)",
        &Environment::new(),
        &context,
    )
    .unwrap();
    assert_eq!(function_calls, Value::Integer(241));

    let closure_calls = evaluate(
        "{ let advance = fn(start: int) -> int effect local { \
         var current = start; set current = current + 1; current }; \
         advance(1) * 100 + advance(40) }",
        &Environment::new(),
    )
    .unwrap();
    assert_eq!(closure_calls, Value::Integer(241));
}
