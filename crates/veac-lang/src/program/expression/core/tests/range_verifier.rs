use super::{raw, raw_with_types, verify_error};
use crate::program::expression::{
    CoreInstruction, CoreInstructionKind, FunctionMap, InputId, PrimitiveType, TypeEnvironment,
    Value, ValueType,
};

#[test]
fn range_core_preserves_source_order_and_optional_step() {
    let (default, _) = raw("1 .. 5");
    let range = instruction(&default);
    let CoreInstructionKind::Range { start, end, step } = &range.kind else {
        unreachable!()
    };
    assert_eq!(start.value(), 0);
    assert_eq!(end.value(), 1);
    assert_eq!(*step, None);

    let (explicit, _) = raw("1 .. 5 by 2");
    let CoreInstructionKind::Range { start, end, step } = &instruction(&explicit).kind else {
        unreachable!()
    };
    assert_eq!(
        (start.value(), end.value(), step.unwrap().value()),
        (0, 1, 2)
    );
}

#[test]
fn rejects_range_result_and_operand_type_corruption() {
    let (mut result, functions) = raw("0 .. 2");
    let integer_type = result.blocks[0].instructions[0].type_id;
    instruction_mut(&mut result).type_id = integer_type;
    assert_message(
        result,
        &functions,
        "Range instruction must declare range<int>",
    );

    let (mut operand, functions) = raw("{ let wrong = \"x\"; 0 .. 2 by 1 }");
    let wrong = operand.blocks[0]
        .instructions
        .iter()
        .find(|value| matches!(value.kind, CoreInstructionKind::Literal(Value::Text(_))))
        .unwrap()
        .id;
    let CoreInstructionKind::Range { step, .. } = &mut instruction_mut(&mut operand).kind else {
        unreachable!()
    };
    *step = Some(wrong);
    assert_message(operand, &functions, "Range operands must have type int");
}

#[test]
fn range_metadata_separates_shape_from_leaf_dependencies() {
    let types = ["start", "end", "step"]
        .into_iter()
        .map(|name| {
            (
                name.to_owned(),
                ValueType::primitive(PrimitiveType::Integer),
            )
        })
        .collect::<TypeEnvironment>();
    let (mut program, functions) = raw_with_types("start .. end by step", &types);
    let metadata = &instruction(&program).metadata;
    assert_eq!(
        metadata.shape_dependencies.shape_input_ids(),
        [InputId::new(0), InputId::new(1), InputId::new(2)]
    );
    assert_eq!(
        metadata.shape_dependencies.leaf_input_ids(),
        [InputId::new(0), InputId::new(1), InputId::new(2)]
    );
    assert_eq!(
        metadata.leaf_dependencies.shape_input_ids(),
        [InputId::new(0), InputId::new(2)]
    );
    assert_eq!(
        metadata.leaf_dependencies.leaf_input_ids(),
        [InputId::new(0), InputId::new(2)]
    );

    let end = program.blocks[0].instructions[1].metadata.clone();
    instruction_mut(&mut program).metadata.absorb_leaf(&end);
    assert_message(
        program,
        &functions,
        "instruction metadata does not match its operands",
    );
}

fn instruction(program: &super::super::CoreProgram) -> &CoreInstruction {
    program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Range { .. }))
        .unwrap()
}

fn instruction_mut(program: &mut super::super::CoreProgram) -> &mut CoreInstruction {
    program
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Range { .. }))
        .unwrap()
}

fn assert_message(program: super::super::CoreProgram, functions: &FunctionMap, message: &str) {
    assert_eq!(verify_error(program, functions).message(), message);
}
