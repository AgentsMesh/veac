use super::{raw, raw_with_types, verify_error};
use crate::program::expression::{
    CoreInstruction, CoreInstructionKind, FunctionMap, PrimitiveType, Stage, TypeEnvironment,
    Value, ValueId, ValueType,
};

#[test]
fn rejects_list_and_tuple_contract_corruption() {
    let (mut list, functions) = raw("{ let wrong = \"x\"; [1, 2] }");
    let wrong = literal_id(&list, |value| matches!(value, Value::Text(_)));
    let CoreInstructionKind::List { elements } = &mut instruction_mut(&mut list, |kind| {
        matches!(kind, CoreInstructionKind::List { .. })
    })
    .kind
    else {
        unreachable!()
    };
    elements[0] = wrong;
    assert_message(list, &functions, "collection element type does not match");

    let (mut tuple, functions) = raw("(1, \"x\")");
    let CoreInstructionKind::Tuple { elements } = &mut instruction_mut(&mut tuple, |kind| {
        matches!(kind, CoreInstructionKind::Tuple { .. })
    })
    .kind
    else {
        unreachable!()
    };
    elements.pop();
    assert_message(
        tuple,
        &functions,
        "tuple instruction arity does not match its type",
    );
}

#[test]
fn rejects_map_key_value_and_token_type_corruption() {
    let (mut key, functions) = raw("{ let wrong = 1; #{\"a\": 2} }");
    let wrong = key.blocks[0].instructions[0].id;
    let CoreInstructionKind::MapKey { key: operand, .. } = &mut instruction_mut(&mut key, |kind| {
        matches!(kind, CoreInstructionKind::MapKey { .. })
    })
    .kind
    else {
        unreachable!()
    };
    *operand = wrong;
    assert_message(
        key,
        &functions,
        "MapKey operand does not match the map key type",
    );

    let (mut value, functions) = raw("{ let wrong = \"x\"; #{\"a\": 2} }");
    let wrong = value.blocks[0].instructions[0].id;
    let CoreInstructionKind::MapValue { value: operand, .. } =
        &mut instruction_mut(&mut value, |kind| {
            matches!(kind, CoreInstructionKind::MapValue { .. })
        })
        .kind
    else {
        unreachable!()
    };
    *operand = wrong;
    assert_message(
        value,
        &functions,
        "MapValue operand does not match the map value type",
    );

    let (mut token, functions) = raw("#{\"a\": 1}");
    let pending_type = instruction(&token, |kind| {
        matches!(kind, CoreInstructionKind::MapKey { .. })
    })
    .type_id;
    instruction_mut(&mut token, |kind| {
        matches!(kind, CoreInstructionKind::MapValue { .. })
    })
    .type_id = pending_type;
    assert_message(
        token,
        &functions,
        "map instruction declares the wrong token type",
    );
}

#[test]
fn rejects_structural_shape_and_leaf_metadata_corruption() {
    let types = [(
        "input".to_owned(),
        ValueType::primitive(PrimitiveType::Integer),
    )]
    .into_iter()
    .collect::<TypeEnvironment>();
    let (mut list, functions) = raw_with_types("[input]", &types);
    instruction_mut(&mut list, |kind| {
        matches!(kind, CoreInstructionKind::List { .. })
    })
    .metadata
    .shape_stage = Stage::Build;
    assert_message(
        list,
        &functions,
        "instruction metadata does not match its operands",
    );
}

fn literal_id(program: &super::super::CoreProgram, predicate: impl Fn(&Value) -> bool) -> ValueId {
    let instruction = instruction(
        program,
        |kind| matches!(kind, CoreInstructionKind::Literal(value) if predicate(value)),
    );
    instruction.id
}

fn instruction(
    program: &super::super::CoreProgram,
    predicate: impl Fn(&CoreInstructionKind) -> bool,
) -> &CoreInstruction {
    program
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .find(|instruction| predicate(&instruction.kind))
        .unwrap()
}

fn instruction_mut(
    program: &mut super::super::CoreProgram,
    predicate: impl Fn(&CoreInstructionKind) -> bool,
) -> &mut CoreInstruction {
    program
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|instruction| predicate(&instruction.kind))
        .unwrap()
}

fn assert_message(program: super::super::CoreProgram, functions: &FunctionMap, message: &str) {
    assert_eq!(verify_error(program, functions).message(), message);
}
