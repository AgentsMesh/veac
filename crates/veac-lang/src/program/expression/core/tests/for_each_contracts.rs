use super::{raw, verify_error};
use crate::program::expression::{BlockId, CoreInstructionKind, CoreTerminator, Value};

fn loop_mut(program: &mut super::super::CoreProgram) -> &mut super::super::CoreForEach {
    program
        .blocks
        .iter_mut()
        .find_map(|block| match &mut block.terminator {
            CoreTerminator::ForEach(value) => Some(value.as_mut()),
            _ => None,
        })
        .unwrap()
}

fn message(
    program: super::super::CoreProgram,
    functions: &crate::program::expression::FunctionMap,
) -> String {
    verify_error(program, functions).message().to_owned()
}

#[test]
fn for_each_rejects_non_iterables_and_bad_continuations() {
    let (base, functions) = raw("for value in [1] { value }");
    let mut scalar = base.clone();
    let iterable = loop_mut(&mut scalar).iterable;
    let integer = scalar.blocks[0]
        .instructions
        .iter()
        .find(|value| matches!(value.kind, CoreInstructionKind::Literal(Value::Integer(_))))
        .unwrap()
        .type_id;
    scalar.blocks[0].instructions[iterable.index().unwrap()].kind =
        CoreInstructionKind::Literal(Value::Integer(1));
    scalar.blocks[0].instructions[iterable.index().unwrap()].type_id = integer;
    scalar.blocks[0].instructions[iterable.index().unwrap()].metadata =
        crate::program::expression::CoreValueMetadata::constant();
    let actual = message(scalar, &functions);
    assert!(actual.contains("not iterable"), "{actual}");

    let mut missing = base.clone();
    loop_mut(&mut missing).continuation = BlockId::new(99);
    assert!(message(missing, &functions).contains("unknown block"));

    let mut no_parameter = base;
    let continuation = loop_mut(&mut no_parameter).continuation.index().unwrap();
    no_parameter.blocks[continuation].parameters.clear();
    assert!(message(no_parameter, &functions).contains("exactly one result parameter"));
}

#[test]
fn for_each_rejects_shared_continuation_and_unsynthesized_body() {
    let source = "if true { for value in [1] { value } } else { [2] }";
    let (base, functions) = raw(source);
    let mut shared = base.clone();
    let continuation = loop_mut(&mut shared).continuation;
    let loop_block = shared
        .blocks
        .iter()
        .position(|block| matches!(block.terminator, CoreTerminator::ForEach(_)))
        .unwrap();
    let alternate = shared
        .blocks
        .iter_mut()
        .enumerate()
        .find(|(index, block)| {
            *index != loop_block
                && matches!(block.terminator, CoreTerminator::Jump { ref arguments, .. } if arguments.len() == 1)
        })
        .unwrap()
        .1;
    let CoreTerminator::Jump { target, .. } = &mut alternate.terminator else {
        unreachable!()
    };
    *target = continuation;
    let actual = message(shared, &functions);
    assert!(actual.contains("exclusive predecessor"), "{actual}");

    let (mut body, functions) = raw("for value in [1] { value }");
    let body_id = loop_mut(&mut body).body.index().unwrap();
    body.closure_definitions[body_id].span = 0..1;
    assert!(message(body, &functions).contains("synthesized effect-checked"));
}

#[test]
fn for_each_accepts_identifier_map_entries_and_uses_source_order() {
    let (program, functions) =
        raw("for entry in #{identifier(\"b\"): 2, identifier(\"a\"): 1} { entry }");
    super::super::verify(program, functions.registry(), &[]).unwrap();
}
