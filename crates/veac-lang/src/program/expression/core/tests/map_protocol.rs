use super::{raw, raw_with_types, verify_error};
use crate::program::expression::{
    CoreInstructionKind, CoreTerminator, FunctionMap, PrimitiveType, TypeEnvironment, ValueType,
};

#[test]
fn rejects_non_dense_ordinals_and_inexact_entry_counts() {
    let (mut ordinal, functions) = raw("#{\"a\": 1, \"b\": 2}");
    let keys = positions(&ordinal, |kind| {
        matches!(kind, CoreInstructionKind::MapKey { .. })
    });
    let CoreInstructionKind::MapKey { ordinal: value, .. } =
        &mut ordinal.blocks[keys[1].0].instructions[keys[1].1].kind
    else {
        unreachable!()
    };
    *value = 0;
    assert_message(
        ordinal,
        &functions,
        "MapKey ordinals must be dense and ordered",
    );

    for (entries, message) in [
        (1, "map builder entry count must end exactly at MapFinish"),
        (3, "map builder must be consumed by MapKey"),
    ] {
        let (mut count, functions) = raw("#{\"a\": 1, \"b\": 2}");
        let begin = positions(&count, |kind| {
            matches!(kind, CoreInstructionKind::MapBegin { .. })
        })[0];
        let CoreInstructionKind::MapBegin { entries: declared } =
            &mut count.blocks[begin.0].instructions[begin.1].kind
        else {
            unreachable!()
        };
        *declared = entries;
        assert_message(count, &functions, message);
    }

    let (mut huge, functions) = raw("#{\"a\": 1}");
    let begin = positions(&huge, |kind| {
        matches!(kind, CoreInstructionKind::MapBegin { .. })
    })[0];
    let CoreInstructionKind::MapBegin { entries } =
        &mut huge.blocks[begin.0].instructions[begin.1].kind
    else {
        unreachable!()
    };
    *entries = u32::MAX;
    assert_message(
        huge,
        &functions,
        "MapBegin entry count exceeds available Core instructions",
    );
}

#[test]
fn rejects_multiple_consumption_and_terminator_escape() {
    let (mut shared, functions) = raw("#{\"a\": 1}");
    let begin = positions(&shared, |kind| {
        matches!(kind, CoreInstructionKind::MapBegin { .. })
    })[0];
    let begin_id = shared.blocks[begin.0].instructions[begin.1].id;
    let finish = positions(&shared, |kind| {
        matches!(kind, CoreInstructionKind::MapFinish { .. })
    })[0];
    let CoreInstructionKind::MapFinish { builder } =
        &mut shared.blocks[finish.0].instructions[finish.1].kind
    else {
        unreachable!()
    };
    *builder = begin_id;
    shared.blocks[finish.0].instructions[finish.1].metadata = shared.blocks[begin.0].instructions
        [begin.1]
        .metadata
        .clone();
    assert_message(
        shared,
        &functions,
        "map token must have exactly one consuming use",
    );

    let (mut escape, functions) = raw("#{\"a\": 1}");
    let begin_id = escape.blocks[0].instructions[0].id;
    let CoreInstructionKind::MapBegin { entries } = &mut escape.blocks[0].instructions[0].kind
    else {
        unreachable!()
    };
    *entries = 0;
    escape.blocks[0].instructions.truncate(1);
    escape.types.entries.pop();
    let CoreTerminator::Return { value, .. } = &mut escape.blocks[0].terminator else {
        unreachable!()
    };
    *value = begin_id;
    assert_message(
        escape,
        &functions,
        "internal map token cannot escape through a terminator",
    );
}

#[test]
fn rejects_consumption_that_does_not_postdominate_definition() {
    let types = [(
        "input".to_owned(),
        ValueType::primitive(PrimitiveType::Boolean),
    )]
    .into_iter()
    .collect::<TypeEnvironment>();
    let source = "if input { #{\"a\": 1} } else { #{} }";
    let (mut program, functions) = raw_with_types(source, &types);
    let begin_index = program.blocks[1]
        .instructions
        .iter()
        .position(|instruction| matches!(instruction.kind, CoreInstructionKind::MapBegin { .. }))
        .unwrap();
    let begin = program.blocks[1].instructions.remove(begin_index);
    program.blocks[0].instructions.push(begin);
    assert_message(
        program,
        &functions,
        "map token consumption must postdominate its definition",
    );
}

fn positions(
    program: &super::super::CoreProgram,
    predicate: impl Fn(&CoreInstructionKind) -> bool,
) -> Vec<(usize, usize)> {
    program
        .blocks
        .iter()
        .enumerate()
        .flat_map(|(block, value)| {
            let predicate = &predicate;
            value
                .instructions
                .iter()
                .enumerate()
                .filter_map(move |(index, instruction)| {
                    predicate(&instruction.kind).then_some((block, index))
                })
        })
        .collect()
}

fn assert_message(program: super::super::CoreProgram, functions: &FunctionMap, message: &str) {
    assert_eq!(verify_error(program, functions).message(), message);
}
