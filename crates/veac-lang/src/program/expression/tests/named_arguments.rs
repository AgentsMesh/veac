use crate::program::expression::{
    compile_expression, compile_functions, evaluate_in, CoreInstructionKind, Environment,
    ExpressionContext, FunctionDefinition, FunctionParameter, TypeEnvironment, Value, ValueType,
};

fn context() -> ExpressionContext {
    let integer = ValueType::primitive(crate::program::expression::PrimitiveType::Integer);
    compile_functions(
        &ExpressionContext::empty(),
        &[FunctionDefinition::new(
            "pair",
            vec![
                FunctionParameter::new("first", integer.clone()),
                FunctionParameter::new("second", integer.clone()),
            ],
            integer,
            "{ first * 10 + second }",
        )],
    )
    .unwrap()
}

#[test]
fn named_user_arguments_bind_by_declaration_slot() {
    let context = context();
    assert_eq!(
        evaluate_in("pair(second: 2, first: 1)", &Environment::new(), &context).unwrap(),
        Value::Integer(12)
    );
    assert_eq!(
        evaluate_in("pair(1, 2)", &Environment::new(), &context).unwrap(),
        Value::Integer(12)
    );
}

#[test]
fn named_argument_expressions_execute_in_source_order() {
    let source = r#"{
      var state = 0;
      let result = pair(
        second: { set state = 2; state },
        first: { set state = 1; state }
      );
      result * 10 + state
    }"#;
    assert_eq!(
        evaluate_in(source, &Environment::new(), &context()).unwrap(),
        Value::Integer(121)
    );
}

#[test]
fn core_evaluates_source_order_but_calls_in_slot_order() {
    let compiled = compile_expression(
        "pair(second: 2, first: 1)",
        &TypeEnvironment::new(),
        &context(),
    )
    .unwrap();
    let instructions = compiled.core().blocks()[0].instructions();
    let literals = instructions
        .iter()
        .filter(|instruction| matches!(instruction.kind(), CoreInstructionKind::Literal(_)))
        .collect::<Vec<_>>();
    assert!(matches!(
        literals[0].kind(),
        CoreInstructionKind::Literal(Value::Integer(2))
    ));
    assert!(matches!(
        literals[1].kind(),
        CoreInstructionKind::Literal(Value::Integer(1))
    ));
    let CoreInstructionKind::Call { arguments, .. } = instructions.last().unwrap().kind() else {
        panic!("expected call")
    };
    assert_eq!(arguments, &[literals[1].id(), literals[0].id()]);
}

#[test]
fn invalid_named_calls_fail_with_specific_codes() {
    let context = context();
    for (source, code) in [
        ("pair(first: 1, 2)", "EXPRESSION_CALL_ARGUMENT_MIXED"),
        (
            "pair(first: 1, other: 2)",
            "EXPRESSION_CALL_ARGUMENT_UNKNOWN",
        ),
        (
            "pair(first: 1, first: 2)",
            "EXPRESSION_CALL_ARGUMENT_DUPLICATE",
        ),
        ("pair(first: 1)", "EXPRESSION_CALL_ARGUMENT_MISSING"),
        (
            "min(left: 1, right: 2)",
            "EXPRESSION_CALL_NAMED_UNSUPPORTED",
        ),
        (
            "(fn(value: int) -> int effect pure { value })(value: 1)",
            "EXPRESSION_CALL_NAMED_UNSUPPORTED",
        ),
        (
            "map(iterable: [1], callback: fn(value: int) -> int effect pure { value })",
            "EXPRESSION_CALL_NAMED_UNSUPPORTED",
        ),
    ] {
        let error = evaluate_in(source, &Environment::new(), &context).unwrap_err();
        assert_eq!(error.code(), code, "{source}: {}", error.message());
    }
}
