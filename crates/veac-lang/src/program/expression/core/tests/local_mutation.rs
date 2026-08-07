use super::{raw, verify_error};
use crate::program::expression::{
    CoreInstructionKind, CoreValueMetadata, LocalSlotId, ValueTypeKind,
};

fn code(program: super::super::CoreProgram) -> &'static str {
    verify_error(program, &crate::program::expression::FunctionMap::new()).code()
}

fn assert_dominance_error(program: super::super::CoreProgram) {
    let error = verify_error(program, &crate::program::expression::FunctionMap::new());
    assert_eq!(error.code(), "EXPRESSION_CORE_VERIFY");
    assert!(error.message().contains("initializer does not dominate"));
}

fn local_position(
    program: &super::super::CoreProgram,
    predicate: impl Fn(&CoreInstructionKind) -> bool,
) -> (usize, usize) {
    program
        .blocks
        .iter()
        .enumerate()
        .find_map(|(block, value)| {
            value
                .instructions
                .iter()
                .position(|instruction| predicate(&instruction.kind))
                .map(|position| (block, position))
        })
        .unwrap()
}

#[test]
fn rejects_unknown_missing_duplicate_and_non_dense_local_slots() {
    let (mut unknown, _) = raw("{ var value = 1; value }");
    unknown.local_slots.clear();
    assert_eq!(code(unknown), "EXPRESSION_CORE_VERIFY");

    let (mut missing, _) = raw("{ var value = 1; set value = 2; value }");
    let (block, position) = local_position(&missing, |kind| {
        matches!(kind, CoreInstructionKind::LocalInit { .. })
    });
    let CoreInstructionKind::LocalInit { slot, value } =
        missing.blocks[block].instructions[position].kind
    else {
        unreachable!()
    };
    missing.blocks[block].instructions[position].kind =
        CoreInstructionKind::LocalSet { slot, value };
    assert_eq!(code(missing), "EXPRESSION_CORE_VERIFY");

    let (mut duplicate, _) = raw("{ var value = 1; set value = 2; value }");
    let (block, position) = local_position(&duplicate, |kind| {
        matches!(kind, CoreInstructionKind::LocalSet { .. })
    });
    let CoreInstructionKind::LocalSet { slot, value } =
        duplicate.blocks[block].instructions[position].kind
    else {
        unreachable!()
    };
    duplicate.blocks[block].instructions[position].kind =
        CoreInstructionKind::LocalInit { slot, value };
    assert_eq!(code(duplicate), "EXPRESSION_CORE_VERIFY");

    let (mut dense, _) = raw("{ var value = 1; value }");
    dense.local_slots[0].id = LocalSlotId::new(1);
    assert_eq!(code(dense), "EXPRESSION_CORE_VERIFY");
}

#[test]
fn rejects_local_type_metadata_and_function_storage_corruption() {
    let (mut typed, _) = raw("{ var value = 1; let flag = true; set value = value + 1; value }");
    let flag = typed.blocks[0]
        .instructions
        .iter()
        .find(|instruction| {
            matches!(
                instruction.kind,
                CoreInstructionKind::Literal(crate::program::expression::Value::Bool(true))
            )
        })
        .unwrap()
        .id;
    let (block, position) = local_position(&typed, |kind| {
        matches!(kind, CoreInstructionKind::LocalSet { .. })
    });
    let CoreInstructionKind::LocalSet { value, .. } =
        &mut typed.blocks[block].instructions[position].kind
    else {
        unreachable!()
    };
    *value = flag;
    assert_eq!(code(typed), "EXPRESSION_CORE_VERIFY");

    let (mut metadata, _) = raw("{ var value = 1; value }");
    metadata.local_slots[0].metadata = CoreValueMetadata::constant();
    assert_eq!(code(metadata), "EXPRESSION_CORE_VERIFY");

    let (mut function, _) =
        raw("{ var value = 1; let callback = fn(v: int) -> int effect pure { v }; value }");
    let function_type = function
        .types
        .entries()
        .iter()
        .find(|entry| {
            matches!(
                entry.kind().value_type().map(|value| value.kind()),
                Some(ValueTypeKind::Function { .. })
            )
        })
        .unwrap()
        .id();
    function.local_slots[0].type_id = function_type;
    assert_eq!(code(function), "EXPRESSION_CORE_VERIFY");
}

#[test]
fn rejects_initializer_that_does_not_dominate_every_access() {
    let (mut program, _) = raw("{ var value = 0; if true { set value = 1; value } \
         else { set value = 2; value } }");
    let (block, position) = local_position(&program, |kind| {
        matches!(kind, CoreInstructionKind::LocalInit { .. })
    });
    let initializer = program.blocks[block].instructions.remove(position);
    program.blocks[1].instructions.insert(0, initializer);
    assert_dominance_error(program);
}

#[test]
fn rejects_same_block_access_before_initialization() {
    let (mut program, _) = raw("{ var value = 1; value }");
    let (block, initialize) = local_position(&program, |kind| {
        matches!(kind, CoreInstructionKind::LocalInit { .. })
    });
    let (_, read) = local_position(&program, |kind| {
        matches!(kind, CoreInstructionKind::LocalGet { .. })
    });
    let initializer = program.blocks[block].instructions[initialize].kind.clone();
    program.blocks[block].instructions[initialize].kind =
        program.blocks[block].instructions[read].kind.clone();
    program.blocks[block].instructions[read].kind = initializer;
    assert_dominance_error(program);

    let (mut program, _) = raw("{ var value = 1; set value = 2; value }");
    let (block, initialize) = local_position(&program, |kind| {
        matches!(kind, CoreInstructionKind::LocalInit { .. })
    });
    let (_, assign) = local_position(&program, |kind| {
        matches!(kind, CoreInstructionKind::LocalSet { .. })
    });
    let CoreInstructionKind::LocalInit { slot, value } =
        program.blocks[block].instructions[initialize].kind
    else {
        unreachable!()
    };
    let CoreInstructionKind::LocalSet {
        value: assigned, ..
    } = program.blocks[block].instructions[assign].kind
    else {
        unreachable!()
    };
    program.blocks[block].instructions[initialize].kind =
        CoreInstructionKind::LocalSet { slot, value };
    program.blocks[block].instructions[assign].kind = CoreInstructionKind::LocalInit {
        slot,
        value: assigned,
    };
    assert_dominance_error(program);
}
