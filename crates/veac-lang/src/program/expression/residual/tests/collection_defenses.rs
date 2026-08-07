use super::support::*;
use crate::program::expression::{
    CompiledExpression, CoreInstruction, CoreInstructionKind, CoreTerminator, CoreTypeId, Value,
    ValueId,
};

#[test]
fn collection_callbacks_must_be_materialized_and_non_escaping() {
    let mut missing = compile("map([1], fn(value: int) -> int effect pure { value })", &[]);
    let callable = collection_callable(&missing);
    instruction_mut(&mut missing, callable).kind = CoreInstructionKind::Literal(Value::Integer(0));
    assert_contract(&missing, "missing_callback", "callback is unavailable");

    let mut escaping = compile("map([1], fn(value: int) -> int effect pure { value })", &[]);
    let callable = collection_callable(&escaping);
    let CoreInstructionKind::Collection { iterable, .. } = &mut collection_mut(&mut escaping).kind
    else {
        unreachable!()
    };
    *iterable = callable;
    assert_eq!(
        failure(&escaping, "escaping_callback").code(),
        "RESIDUAL_INSTRUCTION_UNSUPPORTED"
    );

    let mut capture = compile(
        "{ let offset = 1; map([1], fn(value: int) -> int effect pure { value + offset }) }",
        &[],
    );
    let callable = collection_callable(&capture);
    let CoreInstructionKind::Closure { captures, .. } =
        &mut instruction_mut(&mut capture, callable).kind
    else {
        unreachable!()
    };
    captures[0] = ValueId::new(u32::MAX);
    assert_contract(&capture, "missing_callback_capture", "Core value");

    let mut returned = compile("map([1], fn(value: int) -> int effect pure { value })", &[]);
    let callable = collection_callable(&returned);
    let CoreTerminator::Return { value, .. } =
        &mut returned.program.core_mut().blocks[0].terminator
    else {
        unreachable!()
    };
    *value = callable;
    assert_eq!(
        failure(&returned, "returned_callback").code(),
        "RESIDUAL_INSTRUCTION_UNSUPPORTED"
    );
}

#[test]
fn collection_initial_predicate_and_result_contracts_fail_closed() {
    let mut fold = compile(
        "fold([1], 0, fn(total: int, value: int) -> int effect pure { total + value })",
        &[],
    );
    let CoreInstructionKind::Collection { initial, .. } = &mut collection_mut(&mut fold).kind
    else {
        unreachable!()
    };
    *initial = None;
    assert_contract(&fold, "missing_initial", "initial value is unavailable");

    let mut result = compile("map([1], fn(value: int) -> int effect pure { value })", &[]);
    collection_mut(&mut result).type_id = CoreTypeId::new(u32::MAX);
    assert_contract(&result, "missing_result_type", "result type is unavailable");

    let mut predicate = compile(
        "filter([1], fn(value: int) -> bool effect pure { true })",
        &[],
    );
    let body = &mut predicate.program.core_mut().closure_definitions[0].body;
    let literal = body.blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::Literal(Value::Bool(_))))
        .unwrap();
    literal.kind = CoreInstructionKind::Literal(Value::Integer(1));
    assert_contract(
        &predicate,
        "non_bool_filter",
        "predicate is not Build-stage bool",
    );
}

#[test]
fn sequence_contracts_reject_missing_types_and_forged_elements() {
    let input = [(
        "clock",
        parameter("sequence_clock", veac_ir::TemporalType::Scalar),
    )];
    let mut missing_type = compile("[clock]", &input);
    sequence_mut(&mut missing_type).type_id = CoreTypeId::new(u32::MAX);
    assert_contract(
        &missing_type,
        "missing_sequence_type",
        "sequence result type is unavailable",
    );

    let mut missing_list_type = compile("[1]", &[]);
    sequence_mut(&mut missing_list_type).type_id = CoreTypeId::new(u32::MAX);
    assert_contract(
        &missing_list_type,
        "missing_concrete_list_type",
        "list result type is unavailable",
    );

    let mut forged = compile("[1]", &[]);
    let element = match sequence(&forged).kind() {
        CoreInstructionKind::List { elements } => elements[0],
        _ => unreachable!(),
    };
    instruction_mut(&mut forged, element).kind =
        CoreInstructionKind::Literal(Value::Text("wrong".into()));
    let error = failure(&forged, "forged_list_element");
    assert_eq!(error.code(), "VALUE_LIST_ELEMENT_TYPE");
}

fn collection_callable(expression: &CompiledExpression) -> ValueId {
    let CoreInstructionKind::Collection { callable, .. } =
        collection_instruction(expression).kind()
    else {
        unreachable!()
    };
    *callable
}

fn collection_instruction(expression: &CompiledExpression) -> &CoreInstruction {
    expression
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|value| matches!(value.kind(), CoreInstructionKind::Collection { .. }))
        .unwrap()
}

fn collection_mut(expression: &mut CompiledExpression) -> &mut CoreInstruction {
    expression
        .program
        .core_mut()
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Collection { .. }))
        .unwrap()
}

fn sequence(expression: &CompiledExpression) -> &CoreInstruction {
    expression.core().blocks()[0]
        .instructions()
        .iter()
        .find(|value| matches!(value.kind(), CoreInstructionKind::List { .. }))
        .unwrap()
}

fn sequence_mut(expression: &mut CompiledExpression) -> &mut CoreInstruction {
    expression.program.core_mut().blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::List { .. }))
        .unwrap()
}

fn instruction_mut(expression: &mut CompiledExpression, id: ValueId) -> &mut CoreInstruction {
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
