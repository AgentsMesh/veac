use super::*;
use crate::program::expression::core::metadata::EffectEvidence;
use crate::program::expression::core::verify::definitions::Definitions;
use crate::program::expression::core::verify::test_support::raw;
use crate::program::expression::{CoreInstruction, CoreTypeId, Effect};

const MAP: &str = "map([1, 2], fn(value: int) -> int effect pure { value + 1 })";

fn collection(program: &CoreProgram) -> CoreInstruction {
    program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Collection { .. }))
        .unwrap()
        .clone()
}

fn callable(value: &CoreInstruction) -> ValueId {
    let CoreInstructionKind::Collection { callable, .. } = &value.kind else {
        unreachable!()
    };
    *callable
}

#[test]
fn expected_rejects_missing_result_and_callable_metadata() {
    let (program, _) = raw(MAP);
    let mut value = collection(&program);
    let definitions = Definitions::collect(&program).unwrap();
    value.type_id = CoreTypeId::new(99);
    assert!(
        expected(CollectionOperation::Map, &value, &program, &definitions)
            .unwrap_err()
            .message()
            .contains("result must have a value type")
    );

    let mut missing = program.clone();
    let callback = callable(&collection(&missing));
    missing
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| value.id == callback)
        .unwrap()
        .metadata
        .callable = None;
    let definitions = Definitions::collect(&missing).unwrap();
    assert!(expected(
        CollectionOperation::Map,
        &collection(&missing),
        &missing,
        &definitions
    )
    .unwrap_err()
    .message()
    .contains("lacks callable metadata"));
}

#[test]
fn callable_result_requires_an_available_function_type() {
    let (program, _) = raw(MAP);
    let value = collection(&program);
    let definitions = Definitions::collect(&program).unwrap();
    let literal = program.blocks[0].instructions[0].id;
    assert!(callable_result(&program, &definitions, literal, &value)
        .unwrap_err()
        .message()
        .contains("function type"));

    let mut missing = program.clone();
    let callback = callable(&value);
    missing
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| value.id == callback)
        .unwrap()
        .type_id = CoreTypeId::new(99);
    let definitions = Definitions::collect(&missing).unwrap();
    assert!(callable_result(&missing, &definitions, callback, &value)
        .unwrap_err()
        .message()
        .contains("type is unavailable"));
}

#[test]
fn effect_and_topology_evidence_fail_closed() {
    let (program, _) = raw(MAP);
    let value = collection(&program);
    assert!(verify_effect(
        CollectionOperation::Map,
        EffectEvidence::from_effect(Effect::LocalMutation),
        &value
    )
    .is_err());

    let mut temporal = program;
    let iterable = match &collection(&temporal).kind {
        CoreInstructionKind::Collection { iterable, .. } => *iterable,
        _ => unreachable!(),
    };
    temporal
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| value.id == iterable)
        .unwrap()
        .metadata
        .shape_stage = Stage::Temporal;
    let definitions = Definitions::collect(&temporal).unwrap();
    assert!(expected(
        CollectionOperation::Map,
        &collection(&temporal),
        &temporal,
        &definitions
    )
    .unwrap_err()
    .message()
    .contains("topology"));
}
