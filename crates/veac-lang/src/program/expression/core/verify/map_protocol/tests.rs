use super::*;
use crate::program::expression::core::verify::test_support::raw;

#[test]
fn internal_tokens_must_be_rooted_in_a_begin_chain() {
    let (mut program, _) = raw("#{\"a\": 1}");
    let begin = program.blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::MapBegin { .. }))
        .unwrap();
    begin.kind = CoreInstructionKind::MapFinish { builder: begin.id };
    assert!(verify(&program)
        .unwrap_err()
        .message()
        .contains("not rooted in a MapBegin"));
}

#[test]
fn pending_keys_must_be_consumed_by_map_values() {
    let (mut program, _) = raw("#{\"a\": 1}");
    let pending = program.blocks[0]
        .instructions
        .iter()
        .find(|value| matches!(value.kind, CoreInstructionKind::MapKey { .. }))
        .unwrap()
        .id;
    let value = program.blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::MapValue { .. }))
        .unwrap();
    value.kind = CoreInstructionKind::MapFinish { builder: pending };
    assert!(verify(&program)
        .unwrap_err()
        .message()
        .contains("pending map key must be consumed by MapValue"));
}

#[test]
fn a_token_cannot_be_marked_by_two_construction_chains() {
    let (program, _) = raw("#{\"a\": 1}");
    let value = program.blocks[0].instructions[0].clone();
    let mut visited = vec![false; program.value_count()];
    assert!(mark(value.id, &mut visited, &value).is_ok());
    assert!(mark(value.id, &mut visited, &value)
        .unwrap_err()
        .message()
        .contains("more than one construction chain"));
}
