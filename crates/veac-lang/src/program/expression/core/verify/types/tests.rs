use super::*;
use crate::program::expression::{
    compile_functions, ArithmeticOperator, BuiltinFunction, CoreUnaryOperator, ExpressionContext,
    FunctionDefinition, FunctionParameter, PrimitiveType, ValueType,
};

fn value(kind: PrimitiveType) -> ValueType {
    ValueType::primitive(kind)
}

#[test]
fn unary_comparison_and_equality_reject_invalid_operands() {
    let integer = value(PrimitiveType::Integer);
    let boolean = value(PrimitiveType::Boolean);
    assert_eq!(
        unary(CoreUnaryOperator::Not, &boolean, 0..1).unwrap(),
        boolean
    );
    assert_eq!(
        unary(CoreUnaryOperator::Negative, &integer, 0..1).unwrap(),
        integer
    );
    assert!(unary(CoreUnaryOperator::Not, &integer, 0..1).is_err());
    assert!(comparison(&integer, &integer, 0..1).is_ok());
    assert!(comparison(&integer, &boolean, 0..1).is_err());
    assert!(equality(&integer, &integer, &Default::default(), 0..1).is_ok());
    let callable = ValueType::function(
        vec![integer.clone()],
        integer,
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    assert!(equality(&callable, &callable, &Default::default(), 0..1).is_err());
}

#[test]
fn arithmetic_covers_valid_dimensions_and_rejects_invalid_ones() {
    let integer = value(PrimitiveType::Integer);
    let scalar = value(PrimitiveType::Scalar);
    let time = value(PrimitiveType::Time);
    let percent = value(PrimitiveType::Percent);
    let text = value(PrimitiveType::Text);
    assert_eq!(
        arithmetic(ArithmeticOperator::Add, &text, &text, 0..1).unwrap(),
        text
    );
    assert_eq!(
        arithmetic(ArithmeticOperator::Subtract, &time, &time, 0..1).unwrap(),
        time
    );
    assert_eq!(
        arithmetic(ArithmeticOperator::Multiply, &scalar, &time, 0..1).unwrap(),
        time
    );
    assert_eq!(
        arithmetic(ArithmeticOperator::Multiply, &percent, &percent, 0..1).unwrap(),
        percent
    );
    assert!(arithmetic(ArithmeticOperator::Multiply, &integer, &scalar, 0..1).is_err());
    assert!(arithmetic(ArithmeticOperator::Multiply, &time, &time, 0..1).is_err());
    assert_eq!(
        arithmetic(ArithmeticOperator::Divide, &time, &percent, 0..1).unwrap(),
        time
    );
    assert_eq!(
        arithmetic(ArithmeticOperator::Divide, &time, &time, 0..1).unwrap(),
        scalar
    );
    assert!(arithmetic(ArithmeticOperator::Divide, &time, &integer, 0..1).is_err());
    assert!(arithmetic(ArithmeticOperator::Add, &time, &scalar, 0..1).is_err());
}

#[test]
fn builtins_and_user_calls_enforce_exact_signatures() {
    let integer = value(PrimitiveType::Integer);
    let text = value(PrimitiveType::Text);
    assert!(builtin(BuiltinFunction::Min, std::slice::from_ref(&integer), 0..1).is_err());
    assert_eq!(
        builtin(
            BuiltinFunction::Max,
            &[integer.clone(), integer.clone()],
            0..1
        )
        .unwrap(),
        integer
    );
    assert!(builtin(
        BuiltinFunction::Clamp,
        &[integer.clone(), text.clone(), integer.clone()],
        0..1
    )
    .is_err());
    assert!(builtin(BuiltinFunction::Identifier, &[text], 0..1).is_ok());
    assert!(builtin(
        BuiltinFunction::Identifier,
        std::slice::from_ref(&integer),
        0..1
    )
    .is_err());

    let functions = compile_functions(
        &ExpressionContext::empty(),
        &[FunctionDefinition::new(
            "identity",
            vec![FunctionParameter::new("value", integer.clone())],
            integer.clone(),
            "{ value }",
        )],
    )
    .unwrap();
    let function = functions.functions().lookup("identity").unwrap();
    assert!(user_call(function, &[integer], 0..1).is_ok());
    assert!(user_call(function, &[], 0..1).is_err());
}
