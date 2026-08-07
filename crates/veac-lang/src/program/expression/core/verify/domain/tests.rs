use super::*;
use crate::program::expression::core::verify::definitions::Definitions;
use crate::program::expression::core::verify::test_support::raw;
use crate::program::expression::{
    compile_expression, compile_functions, CoreCallTarget, CoreInstructionKind, CoreType,
    CoreTypeEntry, CoreTypeId, ExpressionContext, FunctionDefinition, TypeEnvironment, ValueType,
};
use crate::program::{DomainOperationId, DomainType};

mod coverage;

#[test]
fn instruction_rejects_non_domain_kinds_and_missing_result_types() {
    let (program, _) = raw("1");
    let definitions = Definitions::collect(&program).unwrap();
    let mut probe = program.blocks[0].instructions[0].clone();
    assert!(instruction(
        &probe,
        &program,
        &definitions,
        &DomainOperationRegistry::standard()
    )
    .unwrap_err()
    .message()
    .contains("not a domain operation"));

    probe.kind = CoreInstructionKind::DomainConstruct {
        opcode: DomainOperationId::InterpolationHold.opcode(),
        operands: Vec::new(),
    };
    probe.type_id = CoreTypeId::new(99);
    assert!(instruction(
        &probe,
        &program,
        &definitions,
        &DomainOperationRegistry::standard()
    )
    .unwrap_err()
    .message()
    .contains("result must have a value type"));
}

#[test]
fn instruction_rejects_temporal_topology_from_exact_metadata() {
    let (mut program, _) = raw("(1080px, 1920px)");
    program.blocks[0].instructions[0].metadata.shape_stage = Stage::Temporal;
    let operands = vec![
        program.blocks[0].instructions[0].id,
        program.blocks[0].instructions[1].id,
    ];
    let type_id = CoreTypeId::new(program.types.entries.len() as u32);
    program.types.entries.push(CoreTypeEntry {
        id: type_id,
        kind: CoreType::Value(ValueType::domain(DomainType::Canvas)),
    });
    let definitions = Definitions::collect(&program).unwrap();
    let registry = DomainOperationRegistry::standard();
    let contract = registry.lookup(DomainOperationId::Canvas).unwrap();
    let mut probe = program.blocks[0].instructions[2].clone();
    probe.kind = CoreInstructionKind::DomainConstruct {
        opcode: DomainOperationId::Canvas.opcode(),
        operands: operands.clone(),
    };
    probe.type_id = type_id;
    probe.metadata = expected_metadata(contract, &operands, &definitions);
    assert!(instruction(
        &probe,
        &program,
        &definitions,
        &DomainOperationRegistry::standard()
    )
    .unwrap_err()
    .message()
    .contains("topology"));
}

#[test]
fn called_programs_must_share_the_callers_domain_identity() {
    let compiled = compile_functions(
        &ExpressionContext::empty(),
        &[FunctionDefinition::new(
            "answer",
            Vec::new(),
            crate::program::expression::PrimitiveType::Integer.into(),
            "{ 1 }",
        )],
    )
    .unwrap();
    let context = ExpressionContext::empty().with_functions(compiled.functions().clone());
    let mut caller = compile_expression("answer()", &TypeEnvironment::new(), &context)
        .unwrap()
        .core()
        .clone();
    assert!(caller.blocks[0].instructions.iter().any(|value| matches!(
        value.kind,
        CoreInstructionKind::Call {
            target: CoreCallTarget::User(_),
            ..
        }
    )));
    caller.domain_opset = crate::program::DomainOpsetVersion::from_raw(99);
    assert!(called_identities(&caller, compiled.functions().registry())
        .unwrap_err()
        .message()
        .contains("different domain operation identity"));
}
