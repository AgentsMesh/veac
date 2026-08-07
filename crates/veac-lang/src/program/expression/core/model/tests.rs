use super::*;
use crate::program::expression::{
    compile_expression, ExpressionContext, PrimitiveType, TypeEnvironment,
};

fn raw(source: &str, types: &TypeEnvironment) -> CoreProgram {
    compile_expression(source, types, &ExpressionContext::empty())
        .unwrap()
        .core()
        .clone()
}

#[test]
fn accessors_expose_the_verified_program_without_reconstructing_it() {
    let program = raw("if true { 1 } else { 2 }", &TypeEnvironment::new());
    assert_eq!(program.version(), CORE_VERSION);
    assert_eq!(program.domain_opset(), program.domain_opset);
    assert_eq!(
        program.domain_registry_digest(),
        program.domain_registry_digest
    );
    assert_eq!(program.entry(), program.entry);
    assert_eq!(program.types(), &program.types);
    assert_eq!(program.blocks(), program.blocks.as_slice());
    assert_eq!(program.inputs(), program.inputs.as_slice());
    assert_eq!(
        program.nominal_definitions(),
        program.nominal_definitions.as_slice()
    );
    assert_eq!(program.result_type_id(), program.result_type);
    assert_eq!(
        program.result_type(),
        program.value_type(program.result_type).unwrap()
    );

    for block in &program.blocks {
        assert_eq!(block.id(), block.id);
        assert_eq!(block.instructions(), block.instructions.as_slice());
        assert_eq!(block.parameters(), block.parameters.as_slice());
        assert_eq!(block.terminator(), &block.terminator);
    }
    let parameter = program
        .blocks
        .iter()
        .find_map(|block| block.parameters.first())
        .unwrap();
    assert_eq!(parameter.id(), parameter.id);
    assert_eq!(parameter.type_id(), parameter.type_id);
    assert_eq!(parameter.span(), parameter.span);
    assert_eq!(parameter.metadata(), &parameter.metadata);
    assert_eq!(
        program.value_span(parameter.id),
        Some(parameter.span.clone())
    );

    let instruction = &program.blocks[0].instructions[0];
    assert_eq!(
        program.value_span(instruction.id),
        Some(instruction.span.clone())
    );
    assert_eq!(program.value_span(ValueId::new(u32::MAX)), None);
}

#[test]
fn input_and_closure_collections_report_their_exact_contents() {
    let types = [("seed".to_owned(), PrimitiveType::Integer.into())]
        .into_iter()
        .collect();
    let input = raw("seed", &types);
    assert_eq!(input.inputs().len(), 1);
    assert_eq!(input.inputs()[0].name(), "seed");
    assert_eq!(input.inputs()[0].type_id(), input.inputs()[0].type_id);
    assert!(!input.inputs()[0].trusts_function_value());

    let closure = raw(
        "fn(value: int) -> int effect pure { value }",
        &TypeEnvironment::new(),
    );
    assert_eq!(
        closure.closure_definition_count(),
        closure.closure_definitions().len()
    );
    assert_eq!(closure.closure_definition_count(), 1);
}
