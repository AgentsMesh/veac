use std::collections::BTreeSet;

use super::{
    public, public_at, public_ref, take_builder, take_pending, verify_contract, MapBuilderState,
    MapPendingState, RuntimeValue,
};
use crate::program::expression::core::{CoreInstruction, CoreInstructionKind, CoreType, ValueId};
use crate::program::expression::{compile_expression, ExpressionContext, TypeEnvironment, Value};

fn map_program() -> crate::program::expression::CompiledExpression {
    compile_expression(
        "#{\"key\": 1}",
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap()
}

fn instruction(
    program: &crate::program::expression::CoreProgram,
    predicate: impl Fn(&CoreInstructionKind) -> bool,
) -> CoreInstruction {
    program
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|instruction| predicate(instruction.kind()))
        .unwrap()
        .clone()
}

fn builder(map_type: crate::program::expression::core::CoreTypeId) -> MapBuilderState {
    MapBuilderState {
        map_type,
        expected: 0,
        next: 0,
        entries: Vec::new(),
        keys: BTreeSet::new(),
    }
}

#[test]
fn public_slots_and_consumable_protocol_tokens_fail_closed() {
    let compiled = map_program();
    let literal = instruction(compiled.core(), |kind| {
        matches!(kind, CoreInstructionKind::Literal(Value::Integer(_)))
    });
    let id = ValueId::new(0);
    let values = vec![Some(RuntimeValue::Public(Value::Integer(1)))];
    assert_eq!(public(&values, id, &literal).unwrap(), Value::Integer(1));
    assert_eq!(
        public_ref(&values, id, &literal).unwrap(),
        &Value::Integer(1)
    );
    assert_eq!(public_at(&values, id, &(8..9)).unwrap(), Value::Integer(1));
    assert_eq!(
        public(&values, ValueId::new(9), &literal)
            .unwrap_err()
            .code(),
        "EXPRESSION_RUNTIME_CONTRACT"
    );

    let begin = instruction(compiled.core(), |kind| {
        matches!(kind, CoreInstructionKind::MapBegin { .. })
    });
    let Some(CoreType::MapBuilder { map_type }) = compiled.core().types.get(begin.type_id) else {
        unreachable!()
    };
    let mut builder_slot = vec![Some(RuntimeValue::MapBuilder(builder(*map_type)))];
    assert_eq!(
        take_builder(&mut builder_slot, id, &begin)
            .unwrap()
            .map_type,
        *map_type
    );
    assert!(builder_slot[0].is_none());
    assert_eq!(
        take_builder(&mut builder_slot, id, &begin)
            .err()
            .unwrap()
            .code(),
        "EXPRESSION_RUNTIME_CONTRACT"
    );

    let mut pending_slot = vec![Some(RuntimeValue::MapPending(MapPendingState {
        builder: builder(*map_type),
        key: Value::Text("key".into()),
    }))];
    assert_eq!(
        take_pending(&mut pending_slot, id, &begin).unwrap().key,
        Value::Text("key".into())
    );
    assert!(take_pending(&mut pending_slot, id, &begin).is_err());
}

#[test]
fn runtime_slot_type_contract_checks_public_builder_and_pending_shapes() {
    let compiled = map_program();
    let program = compiled.core();
    let literal = instruction(program, |kind| {
        matches!(kind, CoreInstructionKind::Literal(Value::Integer(_)))
    });
    verify_contract(&RuntimeValue::Public(Value::Integer(1)), &literal, program).unwrap();
    assert!(verify_contract(&RuntimeValue::Public(Value::Bool(true)), &literal, program).is_err());

    let begin = instruction(program, |kind| {
        matches!(kind, CoreInstructionKind::MapBegin { .. })
    });
    let Some(CoreType::MapBuilder { map_type }) = program.types.get(begin.type_id) else {
        unreachable!()
    };
    verify_contract(
        &RuntimeValue::MapBuilder(builder(*map_type)),
        &begin,
        program,
    )
    .unwrap();
    assert!(verify_contract(&RuntimeValue::Public(Value::Integer(1)), &begin, program).is_err());

    let key = instruction(program, |kind| {
        matches!(kind, CoreInstructionKind::MapKey { .. })
    });
    verify_contract(
        &RuntimeValue::MapPending(MapPendingState {
            builder: builder(*map_type),
            key: Value::Text("key".into()),
        }),
        &key,
        program,
    )
    .unwrap();
}
