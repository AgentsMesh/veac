use super::super::*;
use crate::program::expression::core::verify::definitions::Definitions;
use crate::program::expression::core::verify::test_support::raw;
use crate::program::expression::{
    compile_expression, compile_functions, CoreCallTarget, CoreInstruction, CoreInstructionKind,
    CoreType, ExpressionContext, FunctionDefinition, FunctionId, PrimitiveType, TypeEnvironment,
    ValueType,
};
use crate::program::{DomainOperationId, DomainOperationRegistry, DomainType};

fn canvas_program() -> (CoreProgram, CoreInstruction) {
    let program = compile_expression(
        "canvas(1080px, 1920px)",
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap()
    .core()
    .clone();
    let instruction = program.blocks[0]
        .instructions
        .iter()
        .find(|value| matches!(value.kind, CoreInstructionKind::DomainConstruct { .. }))
        .unwrap()
        .clone();
    (program, instruction)
}

fn verify(program: &CoreProgram, instruction: &CoreInstruction) -> Result<ValueType, String> {
    let definitions = Definitions::collect(program).unwrap();
    super::instruction(
        instruction,
        program,
        &definitions,
        &DomainOperationRegistry::standard(),
    )
    .map_err(|error| error.message().to_owned())
}

#[test]
fn domain_identity_rejects_opset_and_registry_digest_drift() {
    let (mut program, _) = raw("1");
    let registry = DomainOperationRegistry::standard();
    assert!(identity(&program, &registry).is_ok());
    program.domain_opset = crate::program::DomainOpsetVersion::from_raw(99);
    assert!(identity(&program, &registry)
        .unwrap_err()
        .message()
        .contains("opset"));
    program.domain_opset = registry.version();
    program.domain_registry_digest = crate::program::DomainRegistryDigest::from_bytes([7; 32]);
    assert!(identity(&program, &registry)
        .unwrap_err()
        .message()
        .contains("digest"));
}

#[test]
fn called_identity_skips_unknown_functions_and_accepts_matching_bodies() {
    let compiled = compile_functions(
        &ExpressionContext::empty(),
        &[FunctionDefinition::new(
            "answer",
            Vec::new(),
            PrimitiveType::Integer.into(),
            "{ 1 }",
        )],
    )
    .unwrap();
    let context = ExpressionContext::empty().with_functions(compiled.functions().clone());
    let mut caller = compile_expression("answer()", &TypeEnvironment::new(), &context)
        .unwrap()
        .core()
        .clone();
    assert!(called_identities(&caller, compiled.functions().registry()).is_ok());
    let call = caller.blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::Call { .. }))
        .unwrap();
    let CoreInstructionKind::Call { target, .. } = &mut call.kind else {
        unreachable!()
    };
    *target = CoreCallTarget::User(FunctionId::from_bytes([0xff; 32]));
    assert!(called_identities(&caller, compiled.functions().registry()).is_ok());
}

#[test]
fn domain_instruction_rejects_unknown_kind_arity_result_and_metadata() {
    let (program, original) = canvas_program();
    assert_eq!(
        verify(&program, &original).unwrap(),
        ValueType::domain(DomainType::Canvas)
    );

    let mut unknown = original.clone();
    unknown.kind = CoreInstructionKind::DomainConstruct {
        opcode: 0xffff,
        operands: Vec::new(),
    };
    assert!(verify(&program, &unknown).unwrap_err().contains("unknown"));

    let mut wrong_kind = original.clone();
    let CoreInstructionKind::DomainConstruct { opcode, operands } = &original.kind else {
        unreachable!()
    };
    wrong_kind.kind = CoreInstructionKind::GraphEmit {
        opcode: *opcode,
        operands: operands.clone(),
    };
    assert!(verify(&program, &wrong_kind)
        .unwrap_err()
        .contains("wrong Core instruction kind"));

    let mut arity = original.clone();
    let CoreInstructionKind::DomainConstruct { operands, .. } = &mut arity.kind else {
        unreachable!()
    };
    operands.pop();
    assert!(verify(&program, &arity).unwrap_err().contains("arity"));

    let mut result = original.clone();
    result.type_id = program.blocks[0].instructions[0].type_id;
    assert!(verify(&program, &result).unwrap_err().contains("declares"));

    let mut metadata = original.clone();
    metadata.metadata = CoreValueMetadata::constant();
    assert!(verify(&program, &metadata)
        .unwrap_err()
        .contains("metadata"));
}

#[test]
fn domain_instruction_rejects_non_value_and_wrong_operand_types() {
    let (mut program, original) = canvas_program();
    let CoreInstructionKind::DomainConstruct { operands, .. } = &original.kind else {
        unreachable!()
    };
    let operand_type = program.blocks[0]
        .instructions
        .iter()
        .find(|value| value.id == operands[0])
        .unwrap()
        .type_id;
    program.types.entries[operand_type.index().unwrap()].kind =
        CoreType::Value(PrimitiveType::Integer.into());
    assert!(verify(&program, &original)
        .unwrap_err()
        .contains("type does not match"));

    program.types.entries[operand_type.index().unwrap()].kind = CoreType::MapBuilder {
        map_type: operand_type,
    };
    assert!(verify(&program, &original)
        .unwrap_err()
        .contains("must have a value type"));

    let registry = DomainOperationRegistry::standard();
    assert!(registry.lookup(DomainOperationId::Canvas).is_some());
}
