use super::{empty_registry, raw};
use crate::program::expression::{CoreInstructionKind, CoreValueMetadata, Effect, Stage, ValueId};
use crate::program::{DomainOperationId, DomainOpsetVersion, DomainType};

const SOURCE: &str = r#"{
    let outgoing = item(identifier("out"), generated_transparent(), during(0s, 2s));
    let incoming = item(identifier("in"), generated_transparent(), during(1s, 2s));
    transition(identifier("cross"), outgoing, incoming, dissolve(1s))
}"#;

fn program() -> super::super::CoreProgram {
    raw(SOURCE).0
}

fn failure(program: super::super::CoreProgram) -> String {
    super::super::verify(program, &empty_registry(), &[])
        .expect_err("mutated relation Core must fail verification")
        .message()
        .to_owned()
}

fn operation(program: &super::super::CoreProgram, opcode: u16) -> usize {
    program.blocks[0]
        .instructions
        .iter()
        .position(|instruction| match instruction.kind() {
            CoreInstructionKind::DomainConstruct { opcode: value, .. }
            | CoreInstructionKind::GraphEmit { opcode: value, .. } => *value == opcode,
            _ => false,
        })
        .unwrap()
}

#[test]
fn raw_relation_core_uses_v5_contract_and_exact_metadata() {
    let program = program();
    assert_eq!(program.domain_opset(), DomainOpsetVersion::V7);
    assert_eq!(
        program.result_type().as_domain(),
        Some(DomainType::Relation)
    );
    let dissolve =
        &program.blocks[0].instructions[operation(&program, DomainOperationId::Dissolve.opcode())];
    assert_eq!(dissolve.metadata().effect(), Effect::Pure);
    assert_eq!(dissolve.metadata().shape_stage(), Stage::Build);
    let relation = &program.blocks[0].instructions
        [operation(&program, DomainOperationId::Transition.opcode())];
    assert_eq!(relation.metadata().effect(), Effect::GraphEmit);
    assert!(matches!(
        relation.kind(),
        CoreInstructionKind::GraphEmit { operands, .. } if operands.len() == 4
    ));
    super::domain_fixture::compiled(program);
}

#[test]
fn raw_relation_core_rejects_opcode_kind_and_operand_type_corruption() {
    let mut opcode = program();
    let index = operation(&opcode, DomainOperationId::Transition.opcode());
    let CoreInstructionKind::GraphEmit { opcode: value, .. } =
        &mut opcode.blocks[0].instructions[index].kind
    else {
        unreachable!()
    };
    *value = 0xffff;
    assert!(failure(opcode).contains("unknown domain operation opcode"));

    let mut kind = program();
    let index = operation(&kind, DomainOperationId::Transition.opcode());
    let operands = match &kind.blocks[0].instructions[index].kind {
        CoreInstructionKind::GraphEmit { operands, .. } => operands.clone(),
        _ => unreachable!(),
    };
    kind.blocks[0].instructions[index].kind = CoreInstructionKind::DomainConstruct {
        opcode: DomainOperationId::Transition.opcode(),
        operands,
    };
    assert!(failure(kind).contains("wrong Core instruction kind"));

    let mut endpoint = program();
    let index = operation(&endpoint, DomainOperationId::Transition.opcode());
    let CoreInstructionKind::GraphEmit { operands, .. } =
        &mut endpoint.blocks[0].instructions[index].kind
    else {
        unreachable!()
    };
    operands[1] = ValueId::new(0);
    assert!(failure(endpoint).contains("operand `from` type"));
}

#[test]
fn raw_relation_core_rejects_mechanism_and_metadata_corruption() {
    let mut mechanism = program();
    let index = operation(&mechanism, DomainOperationId::Transition.opcode());
    let CoreInstructionKind::GraphEmit { operands, .. } =
        &mut mechanism.blocks[0].instructions[index].kind
    else {
        unreachable!()
    };
    operands[3] = operands[1];
    assert!(failure(mechanism).contains("operand `mechanism` type"));

    let mut metadata = program();
    let index = operation(&metadata, DomainOperationId::Transition.opcode());
    metadata.blocks[0].instructions[index].metadata = CoreValueMetadata::constant();
    assert!(failure(metadata).contains("domain instruction metadata"));
}
