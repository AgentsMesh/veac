use crate::program::expression::{
    compile_functions, ExpressionContext, FunctionDefinition, FunctionParameter, PrimitiveType,
    ValueType, MAX_CALL_ARGUMENTS, MAX_FUNCTION_PARAMETERS,
};

fn parameter(name: &str, value_type: ValueType) -> FunctionParameter {
    FunctionParameter::new(name, value_type)
}

fn scalar_type() -> ValueType {
    PrimitiveType::Scalar.into()
}

fn time_type() -> ValueType {
    PrimitiveType::Time.into()
}

fn definition(
    name: &str,
    parameters: Vec<FunctionParameter>,
    return_type: ValueType,
    body: &str,
) -> FunctionDefinition {
    FunctionDefinition::new(name, parameters, return_type, format!("{{{body}}}"))
}

fn code(definitions: &[FunctionDefinition]) -> &'static str {
    compile_functions(&ExpressionContext::empty(), definitions)
        .unwrap_err()
        .code()
}

#[test]
fn function_bodies_are_statically_typed() {
    let scalar = parameter("x", scalar_type());
    for (body, return_type, expected) in [
        ("missing", scalar_type(), "EXPRESSION_UNKNOWN_SYMBOL"),
        ("missing(x)", scalar_type(), "EXPRESSION_UNKNOWN_FUNCTION"),
        ("min(x)", scalar_type(), "EXPRESSION_CALL_ARITY"),
        ("min(x, 1s)", scalar_type(), "EXPRESSION_CALL_ARGUMENT_TYPE"),
        ("x", time_type(), "EXPRESSION_RETURN_TYPE"),
    ] {
        assert_eq!(
            code(&[definition(
                "tested",
                vec![scalar.clone()],
                return_type,
                body
            )]),
            expected,
            "{body}"
        );
    }
}

#[test]
fn user_call_arity_and_types_are_checked_before_execution() {
    let callee = definition(
        "callee",
        vec![parameter("when", time_type())],
        time_type(),
        "when",
    );
    let wrong_arity = definition("caller", vec![], time_type(), "callee()");
    assert_eq!(
        code(&[callee.clone(), wrong_arity]),
        "EXPRESSION_CALL_ARITY"
    );

    let wrong_type = definition("caller", vec![], time_type(), "callee(1)");
    assert_eq!(code(&[callee, wrong_type]), "EXPRESSION_CALL_ARGUMENT_TYPE");
}

#[test]
fn function_bodies_cannot_capture_authoring_environment() {
    let function = definition("capturing", vec![], scalar_type(), "outside + 1");
    assert_eq!(code(&[function]), "EXPRESSION_UNKNOWN_SYMBOL");
}

#[test]
fn direct_and_indirect_cycles_are_rejected() {
    let direct = definition("direct", vec![], scalar_type(), "direct()");
    assert_eq!(code(&[direct]), "EXPRESSION_FUNCTION_CYCLE");

    let first = definition("first", vec![], scalar_type(), "second()");
    let second = definition("second", vec![], scalar_type(), "third()");
    let third = definition("third", vec![], scalar_type(), "first()");
    assert_eq!(code(&[first, second, third]), "EXPRESSION_FUNCTION_CYCLE");
}

#[test]
fn duplicate_functions_and_parameters_fail_closed() {
    let first = definition("same", vec![], scalar_type(), "1");
    let second = definition("same", vec![], scalar_type(), "2");
    assert_eq!(code(&[first, second]), "EXPRESSION_DUPLICATE_FUNCTION");

    let duplicate = definition(
        "duplicate",
        vec![parameter("x", scalar_type()), parameter("x", scalar_type())],
        scalar_type(),
        "x",
    );
    assert_eq!(code(&[duplicate]), "EXPRESSION_DUPLICATE_PARAMETER");
}

#[test]
fn batch_errors_identify_the_owning_function_body() {
    let first = definition("first", vec![], scalar_type(), "1.0");
    let broken = definition("broken", vec![], scalar_type(), "unknown + 1.0");
    let error = compile_functions(&ExpressionContext::empty(), &[first, broken]).unwrap_err();
    assert_eq!(error.function_name(), Some("broken"));
    assert_eq!(error.span(), 1..8);
}

#[test]
fn excessive_function_arity_is_rejected_before_execution() {
    let parameters = (0..=MAX_FUNCTION_PARAMETERS)
        .map(|index| parameter(&format!("p{index}"), scalar_type()))
        .collect();
    let oversized = definition("oversized", parameters, scalar_type(), "p0");
    assert_eq!(code(&[oversized]), "EXPRESSION_FUNCTION_PARAMETER_LIMIT");

    let arguments = vec!["1.0"; MAX_CALL_ARGUMENTS + 1].join(", ");
    let oversized_call = definition(
        "oversized-call",
        vec![],
        scalar_type(),
        &format!("min({arguments})"),
    );
    assert_eq!(code(&[oversized_call]), "EXPRESSION_CALL_ARGUMENT_LIMIT");
}

#[test]
fn maximum_function_arity_remains_compilable() {
    let parameters = (0..MAX_FUNCTION_PARAMETERS)
        .map(|index| parameter(&format!("p{index}"), scalar_type()))
        .collect();
    let callee = definition("callee", parameters, scalar_type(), "p0");
    let arguments = vec!["1.0"; MAX_CALL_ARGUMENTS].join(", ");
    let caller = definition(
        "caller",
        vec![],
        scalar_type(),
        &format!("callee({arguments})"),
    );
    compile_functions(&ExpressionContext::empty(), &[caller, callee]).unwrap();
}
