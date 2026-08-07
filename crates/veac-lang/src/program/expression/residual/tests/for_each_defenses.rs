use super::support::*;
use crate::program::expression::{
    ClosureDefinitionId, CompiledExpression, CoreForEach, CoreInstructionKind, CoreTerminator,
    CoreTypeId, PrimitiveType, ResidualRuntimeValue, Value, ValueId, ValueType,
};

#[test]
fn residual_iterables_and_packing_reject_forged_shapes() {
    let error = super::super::for_each::iterable::ResidualIterable::new(
        ResidualRuntimeValue::Concrete(Value::Integer(1)),
        4..8,
    )
    .err()
    .unwrap();
    assert_eq!(error.code(), "RESIDUAL_CORE_CONTRACT");
    assert!(error.message().contains("not iterable"));

    let scalar = ValueType::primitive(PrimitiveType::Integer);
    let error = super::super::for_each::pack(
        scalar.clone(),
        vec![ResidualRuntimeValue::Concrete(Value::Integer(1))],
        8..12,
    )
    .unwrap_err();
    assert!(error.message().contains("not a list"));

    let error = super::super::for_each::pack(
        ValueType::list(scalar).unwrap(),
        vec![ResidualRuntimeValue::Concrete(Value::Text("wrong".into()))],
        12..16,
    )
    .unwrap_err();
    assert!(error.message().contains("result is invalid"));
}

#[test]
fn for_each_rejects_missing_iterable_capture_and_body_values() {
    let mut iterable = compile("for value in [1] { value }", &[]);
    let iterable_id = loop_value(&iterable).iterable;
    instruction_mut(&mut iterable, iterable_id).kind =
        CoreInstructionKind::Literal(Value::Integer(1));
    assert_contract(&iterable, "forged_iterable", "not iterable");

    let mut capture = compile(
        "{ let offset = 1; for value in [1] { value + offset } }",
        &[],
    );
    loop_value_mut(&mut capture).captures[0] = ValueId::new(u32::MAX);
    assert_contract(&capture, "missing_capture", "Core value");

    let mut body = compile("for value in [1] { value }", &[]);
    loop_value_mut(&mut body).body = ClosureDefinitionId::new(99);
    assert_contract(&body, "missing_body", "body is unavailable");
}

#[test]
fn for_each_rejects_a_missing_continuation_result_type() {
    let mut expression = compile("for value in [1] { value }", &[]);
    let continuation = loop_value(&expression).continuation.index().unwrap();
    expression.program.core_mut().blocks[continuation].parameters[0].type_id =
        CoreTypeId::new(u32::MAX);
    assert_contract(
        &expression,
        "missing_for_each_result_type",
        "result type is unavailable",
    );
}

fn loop_value(expression: &CompiledExpression) -> &CoreForEach {
    expression
        .core()
        .blocks()
        .iter()
        .find_map(|block| match block.terminator() {
            CoreTerminator::ForEach(value) => Some(value),
            _ => None,
        })
        .unwrap()
}

fn loop_value_mut(expression: &mut CompiledExpression) -> &mut CoreForEach {
    expression
        .program
        .core_mut()
        .blocks
        .iter_mut()
        .find_map(|block| match &mut block.terminator {
            CoreTerminator::ForEach(value) => Some(value),
            _ => None,
        })
        .unwrap()
}

fn instruction_mut(
    expression: &mut CompiledExpression,
    id: ValueId,
) -> &mut crate::program::expression::CoreInstruction {
    expression
        .program
        .core_mut()
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| value.id == id)
        .unwrap()
}

fn failure(
    expression: &CompiledExpression,
    name: &str,
) -> crate::program::expression::ResidualizationError {
    super::super::residualize_expression(expression, &no_bindings(), request(name)).unwrap_err()
}

fn assert_contract(expression: &CompiledExpression, name: &str, message: &str) {
    let error = failure(expression, name);
    assert_eq!(error.code(), "RESIDUAL_CORE_CONTRACT");
    assert!(error.message().contains(message), "{}", error.message());
}
