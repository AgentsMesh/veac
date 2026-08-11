use super::domain_fixture::{compiled, result_domain, transform_program};
use super::empty_registry;
use crate::program::expression::{CoreInstructionKind, Effect, Stage, ValueId};
use crate::program::{DomainOperationId, DomainOpsetVersion, DomainType};

fn failure(program: super::super::CoreProgram) -> String {
    super::super::verify(program, &empty_registry(), &[])
        .expect_err("mutated transform Core must fail verification")
        .message()
        .to_owned()
}

#[test]
fn raw_transform_core_uses_v8_numeric_contracts_and_exact_metadata() {
    let program = transform_program();
    assert_eq!(program.domain_opset(), DomainOpsetVersion::V8);
    assert_eq!(result_domain(&program), DomainType::Transform);
    let operations = program.blocks[0]
        .instructions
        .iter()
        .filter(|value| matches!(value.kind(), CoreInstructionKind::DomainConstruct { .. }))
        .collect::<Vec<_>>();
    assert_eq!(operations.len(), 6);
    assert!(operations
        .iter()
        .all(|value| value.metadata().effect() == Effect::Pure));
    assert!(operations
        .iter()
        .all(|value| value.metadata().shape_stage() == Stage::Build));
    assert_eq!(operations[0].metadata().leaf_stage(), Stage::Const);
    assert_eq!(operations[1].metadata().leaf_stage(), Stage::Build);
    compiled(program);
}

#[test]
fn raw_transform_core_rejects_receiver_numeric_and_instruction_corruption() {
    let mut length = transform_program();
    let CoreInstructionKind::DomainConstruct { operands, .. } =
        &mut length.blocks[0].instructions[3].kind
    else {
        unreachable!()
    };
    operands[1] = ValueId::new(0);
    assert!(failure(length).contains("operand `x` type"));

    let mut receiver = transform_program();
    let CoreInstructionKind::DomainConstruct { operands, .. } =
        &mut receiver.blocks[0].instructions[6].kind
    else {
        unreachable!()
    };
    operands[0] = ValueId::new(1);
    assert!(failure(receiver).contains("operand `transform` type"));

    let mut instruction = transform_program();
    instruction.blocks[0].instructions[14].kind = CoreInstructionKind::GraphEmit {
        opcode: DomainOperationId::TransformFlipped.opcode(),
        operands: vec![ValueId::new(11), ValueId::new(12), ValueId::new(13)],
    };
    assert!(failure(instruction).contains("wrong Core instruction kind"));
}

#[test]
fn raw_transform_core_rejects_effect_and_stage_corruption() {
    for metadata in [
        crate::program::expression::CoreValueMetadata::constant(),
        crate::program::expression::CoreValueMetadata::pure(
            Stage::Temporal,
            Default::default(),
            Default::default(),
        ),
    ] {
        let mut program = transform_program();
        program.blocks[0].instructions[3].metadata = metadata;
        assert!(failure(program).contains("domain instruction metadata"));
    }
}
