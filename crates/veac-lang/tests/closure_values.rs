use veac_lang::program::expression::{evaluate, Environment, Value};

fn value(source: &str) -> Value {
    evaluate(source, &Environment::new()).unwrap()
}

fn integer(source: &str) -> i64 {
    let Value::Integer(value) = value(source) else {
        panic!("closure expression must produce an int");
    };
    value
}

fn error(source: &str) -> veac_lang::program::expression::ExpressionError {
    evaluate(source, &Environment::new()).unwrap_err()
}

#[test]
fn closures_are_callable_as_literals_locals_conditionals_and_returned_values() {
    assert_eq!(
        integer("(fn(value: int) -> int effect pure { value + 1 })(41)"),
        42
    );
    assert_eq!(
        integer("{ let add: fn(int) -> int effect pure = fn(value: int) -> int effect pure { value + 2 }; add(40) }",),
        42
    );
    assert_eq!(
        integer(
            "(if true { fn(value: int) -> int effect pure { value + 3 } } else { \
             fn(value: int) -> int effect pure { value } })(39)",
        ),
        42
    );
    assert_eq!(
        integer(
            "{ let make = fn(offset: int) -> fn(int) -> int effect pure effect pure { \
             fn(value: int) -> int effect pure { value + offset } }; make(2)(40) }",
        ),
        42
    );
}

#[test]
fn closure_bodies_are_typed_eagerly_but_execute_only_when_called() {
    let source = "{ let deferred = fn() -> int effect pure { 9223372036854775807 + 1 }; 1 }";
    assert_eq!(integer(source), 1);

    let called =
        "{ let deferred = fn() -> int effect pure { 9223372036854775807 + 1 }; deferred() }";
    let failure = error(called);
    assert_eq!(failure.code(), "EXPRESSION_OVERFLOW");
    assert_eq!(&called[failure.span()], "9223372036854775807 + 1");

    assert_eq!(
        error("fn() -> int effect pure { true }").code(),
        "EXPRESSION_CLOSURE_RETURN_TYPE"
    );
}

#[test]
fn every_closure_parameter_and_return_type_is_required() {
    for (source, expected) in [
        (
            "fn(value) -> int effect pure { value }",
            "EXPRESSION_EXPECTED_TOKEN",
        ),
        ("fn(value: int) { value }", "EXPRESSION_EXPECTED_TOKEN"),
        ("fn(value: int) -> { value }", "EXPRESSION_TYPE_SYNTAX"),
        (
            "fn(value: int) -> int effect pure value",
            "EXPRESSION_EXPECTED_TOKEN",
        ),
    ] {
        assert_eq!(error(source).code(), expected, "{source}");
    }
}

#[test]
fn closure_calls_enforce_callee_arity_and_argument_types() {
    for (source, expected) in [
        (
            "(fn(value: int) -> int effect pure { value })()",
            "EXPRESSION_CALL_ARITY",
        ),
        (
            "(fn(value: int) -> int effect pure { value })(true)",
            "EXPRESSION_CALL_ARGUMENT_TYPE",
        ),
        ("true()", "EXPRESSION_CALL_CALLEE_TYPE"),
        (
            "(if true { 1 } else { 2 })(3)",
            "EXPRESSION_CALL_CALLEE_TYPE",
        ),
    ] {
        assert_eq!(error(source).code(), expected, "{source}");
    }
}

#[test]
fn callee_then_arguments_execute_once_from_left_to_right() {
    let callee =
        "{ let choose = fn(selector: int) -> fn(int, int) -> int effect pure effect pure { \
        if selector > 0 { fn(a: int, b: int) -> int effect pure { a + b } } else { \
        fn(a: int, b: int) -> int effect pure { a - b } } }; \
        choose(9223372036854775807 + 1)(9223372036854775807 * 2, \
        9223372036854775807 + 3) }";
    let first = error(callee);
    assert_eq!(first.code(), "EXPRESSION_OVERFLOW");
    assert_eq!(&callee[first.span()], "9223372036854775807 + 1");

    let arguments = "{ let add = fn(a: int, b: int) -> int effect pure { a + b }; \
        add(9223372036854775807 * 2, 9223372036854775807 + 3) }";
    let first = error(arguments);
    assert_eq!(&arguments[first.span()], "9223372036854775807 * 2");

    let second = arguments.replace("9223372036854775807 * 2", "0");
    let failure = error(&second);
    assert_eq!(&second[failure.span()], "9223372036854775807 + 3");
}

#[test]
fn function_values_have_no_equality_or_ordering_relation() {
    let equality = "{ let same = fn(value: int) -> int effect pure { value }; same == same }";
    assert_eq!(error(equality).code(), "EXPRESSION_FUNCTION_EQUALITY");

    let ordering = "{ let same = fn(value: int) -> int effect pure { value }; same < same }";
    assert_eq!(error(ordering).code(), "EXPRESSION_FUNCTION_ORDERING");
}
