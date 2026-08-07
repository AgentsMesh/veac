use super::*;
use crate::program::expression::core::verify::definitions::Definitions;
use crate::program::expression::core::verify::test_support::raw;
use crate::program::expression::{
    compile_functions, ClosureDefinitionId, CoreInstructionKind, CoreType, CoreTypeEntry,
    CoreTypeId, FunctionDefinition, FunctionMap, FunctionParameter, InputId, PrimitiveType,
    ValueId,
};

fn message(
    program: &CoreProgram,
    value: &CoreInstruction,
    functions: &FunctionMap,
    parameters: &[ValueType],
    captures: &[ValueType],
) -> String {
    let definitions = Definitions::collect(program).unwrap();
    let stages = vec![Stage::Const; parameters.len()];
    instruction(
        value,
        program,
        &definitions,
        functions.registry(),
        parameters,
        &stages,
        captures,
    )
    .unwrap_err()
    .message()
    .to_owned()
}

fn literal_probe() -> (CoreProgram, CoreInstruction, FunctionMap) {
    let (program, functions) = raw("1");
    let probe = program.blocks[0].instructions[0].clone();
    (program, probe, functions)
}

#[test]
fn rejects_unknown_input_parameter_capture_and_closure_metadata() {
    let (program, mut probe, functions) = literal_probe();
    probe.kind = CoreInstructionKind::Input(InputId::new(9));
    assert!(message(&program, &probe, &functions, &[], &[]).contains("unknown Core input"));

    probe.kind = CoreInstructionKind::Parameter(1);
    assert!(message(&program, &probe, &functions, &[], &[]).contains("parameter index"));

    probe.kind = CoreInstructionKind::Capture(1);
    assert!(message(&program, &probe, &functions, &[], &[]).contains("capture index"));

    probe.kind = CoreInstructionKind::Closure {
        definition: ClosureDefinitionId::new(7),
        captures: Vec::new(),
    };
    assert!(message(&program, &probe, &functions, &[], &[]).contains("closure definition"));
}

#[test]
fn invoke_requires_a_value_result_and_callable_contract() {
    let (program, mut probe, functions) = literal_probe();
    probe.kind = CoreInstructionKind::Invoke {
        callee: ValueId::new(0),
        arguments: Vec::new(),
    };
    assert!(message(&program, &probe, &functions, &[], &[]).contains("callable metadata"));

    probe.type_id = CoreTypeId::new(99);
    assert!(message(&program, &probe, &functions, &[], &[]).contains("must have a value type"));
}

#[test]
fn user_call_rejects_an_unresolved_summary_binding() {
    let integer: ValueType = PrimitiveType::Integer.into();
    let compiled = compile_functions(
        &crate::program::expression::ExpressionContext::empty(),
        &[FunctionDefinition::new(
            "identity",
            vec![FunctionParameter::new("value", integer.clone())],
            integer,
            "{ value }",
        )],
    )
    .unwrap();
    let function = compiled.functions().lookup("identity").unwrap();
    let (program, mut probe, _) = literal_probe();
    probe.kind = CoreInstructionKind::Call {
        target: CoreCallTarget::User(function.id()),
        arguments: Vec::new(),
    };
    let definitions = Definitions::collect(&program).unwrap();
    let error = instruction(
        &probe,
        &program,
        &definitions,
        compiled.functions().registry(),
        &[],
        &[],
        &[],
    )
    .unwrap_err();
    assert!(error.message().contains("unresolved metadata binding"));
}

#[test]
fn projection_and_domain_metadata_fail_closed() {
    let (mut program, mut probe, functions) = literal_probe();
    let callable = ValueType::function(
        Vec::new(),
        PrimitiveType::Integer.into(),
        crate::program::expression::FunctionEffect::Pure,
    )
    .unwrap();
    program.types.entries.push(CoreTypeEntry {
        id: CoreTypeId::new(1),
        kind: CoreType::Value(callable),
    });
    probe.kind = CoreInstructionKind::StructProject {
        structure: ValueId::new(0),
        field: crate::program::FieldIndex::new(0),
    };
    probe.type_id = CoreTypeId::new(1);
    assert!(message(&program, &probe, &functions, &[], &[]).contains("closed field metadata"));

    probe.type_id = CoreTypeId::new(99);
    assert!(message(&program, &probe, &functions, &[], &[]).contains("must have a value type"));

    probe.kind = CoreInstructionKind::DomainConstruct {
        opcode: 0,
        operands: Vec::new(),
    };
    assert!(message(&program, &probe, &functions, &[], &[]).contains("versioned operation"));
}

#[test]
fn temporal_shape_is_rejected_even_when_metadata_is_internally_exact() {
    let (mut program, functions) = raw("1 + 2");
    program.blocks[0].instructions[0].metadata.shape_stage = Stage::Temporal;
    let operands = [
        program.blocks[0].instructions[0].metadata.clone(),
        program.blocks[0].instructions[1].metadata.clone(),
    ];
    program.blocks[0].instructions[2].metadata = CoreValueMetadata::combine(operands.iter());
    let probe = program.blocks[0].instructions[2].clone();
    assert!(message(&program, &probe, &functions, &[], &[]).contains("Temporal-stage"));
}
