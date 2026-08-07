use super::{raw_with_types, verify_error};
use crate::program::expression::{
    CoreType, CoreTypeEntry, CoreTypeId, FunctionMap, TypeEnvironment, ValueType,
};

#[test]
fn internal_tokens_cannot_occupy_public_core_positions() {
    let (program, functions, token) = map_input_program("input");

    let mut input = program.clone();
    input.inputs[0].type_id = token;
    assert_rejected(input, &functions);

    let mut result = program.clone();
    result.result_type = token;
    assert_rejected(result, &functions);

    let mut instruction = program;
    instruction.blocks[0].instructions[0].type_id = token;
    assert_rejected(instruction, &functions);

    let (mut parameter, functions, token) = map_input_program("if true { input } else { input }");
    parameter.blocks[3].parameters[0].type_id = token;
    assert_rejected(parameter, &functions);
}

fn map_input_program(source: &str) -> (super::super::CoreProgram, FunctionMap, CoreTypeId) {
    let types = [(
        "input".to_owned(),
        ValueType::parse("map<text, int>").unwrap(),
    )]
    .into_iter()
    .collect::<TypeEnvironment>();
    let (mut program, functions) = raw_with_types(source, &types);
    let map_type = program.result_type;
    let token = CoreTypeId::new(program.types.entries.len() as u32);
    program.types.entries.push(CoreTypeEntry {
        id: token,
        kind: CoreType::MapBuilder { map_type },
    });
    (program, functions, token)
}

fn assert_rejected(program: super::super::CoreProgram, functions: &FunctionMap) {
    assert_eq!(
        verify_error(program, functions).code(),
        "EXPRESSION_CORE_VERIFY"
    );
}
