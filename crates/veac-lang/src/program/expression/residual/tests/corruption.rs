use super::support::*;
use crate::program::expression::{
    BlockId, CoreInputIdentity, CoreInstructionKind, CoreTemporalInputIdentity, CoreTerminator,
    CoreTypeId, InputId, PrimitiveType, ResidualBuildBindings, TypeEnvironment, Value, ValueId,
};
use veac_ir::{TemporalParameterId, TemporalType};

#[test]
fn corrupted_control_references_fail_closed() {
    let mut unknown_block = compile("1", &[]);
    unknown_block.program.core_mut().entry = BlockId::new(99);
    assert_eq!(
        failure(&unknown_block, &no_bindings(), "unknown_block"),
        "RESIDUAL_CORE_CONTRACT"
    );

    let mut unknown_value = compile("1", &[]);
    let CoreTerminator::Return { value, .. } =
        &mut unknown_value.program.core_mut().blocks[0].terminator
    else {
        panic!("fixture must return");
    };
    *value = ValueId::new(99);
    assert_eq!(
        failure(&unknown_value, &no_bindings(), "unknown_value"),
        "RESIDUAL_CORE_CONTRACT"
    );
    assert!(!super::super::effect::branches_are_pure(
        unknown_value.core(),
        [BlockId::new(99), BlockId::new(100)]
    ));
}

#[test]
fn corrupted_branch_values_cannot_escape_verified_types() {
    let build = TypeEnvironment::from([("choose".to_owned(), PrimitiveType::Boolean.into())]);
    let mut concrete = compile_with_build("if choose { 1 } else { 2 }", build, &[]);
    let condition = branch_condition(&concrete);
    concrete.program.core_mut().blocks[0].instructions[condition.index().unwrap()].kind =
        CoreInstructionKind::Literal(Value::Integer(1));
    assert_eq!(
        failure(
            &concrete,
            &binding("choose", Value::Bool(true)),
            "branch_not_bool"
        ),
        "RESIDUAL_CORE_CONTRACT"
    );

    let temporal = [(
        "condition",
        parameter("corrupt_branch", TemporalType::Boolean),
    )];
    let mut wrong_condition = compile("if condition { 1 } else { 2 }", &temporal);
    wrong_condition.program.core_mut().inputs[0].identity =
        CoreInputIdentity::Temporal(CoreTemporalInputIdentity::Parameter {
            parameter_id: TemporalParameterId::new("tpm_corrupt_scalar").unwrap(),
            value_type: TemporalType::Scalar,
        });
    assert_eq!(
        failure(&wrong_condition, &no_bindings(), "temporal_not_bool"),
        "RESIDUAL_CORE_CONTRACT"
    );

    let mut mismatched = compile("if condition { 1 } else { 2 }", &temporal);
    let else_target = branch_targets(&mismatched)[1].index().unwrap();
    mismatched.program.core_mut().blocks[else_target].instructions[0].kind =
        CoreInstructionKind::Literal(Value::Text("wrong".into()));
    assert_eq!(
        failure(&mismatched, &no_bindings(), "branch_type_mismatch"),
        "RESIDUAL_CORE_CONTRACT"
    );
}

#[test]
fn corrupted_domain_identity_and_topology_fail_closed() {
    let mut opcode = compile("during(0s, 1s)", &[]);
    let instruction = opcode.program.core_mut().blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::DomainConstruct { .. }))
        .unwrap();
    let CoreInstructionKind::DomainConstruct {
        opcode: opcode_value,
        ..
    } = &mut instruction.kind
    else {
        unreachable!();
    };
    *opcode_value = u16::MAX;
    assert_eq!(
        failure(&opcode, &no_bindings(), "unknown_opcode"),
        "RESIDUAL_CORE_CONTRACT"
    );

    let types = TypeEnvironment::from([("numerator".to_owned(), PrimitiveType::Integer.into())]);
    let mut topology = compile_with_build("frame_rate(numerator, 1)", types, &[]);
    topology.program.core_mut().inputs[0].identity =
        CoreInputIdentity::Temporal(CoreTemporalInputIdentity::Parameter {
            parameter_id: TemporalParameterId::new("tpm_topology").unwrap(),
            value_type: TemporalType::Integer,
        });
    assert_eq!(
        failure(&topology, &no_bindings(), "temporal_topology"),
        "RESIDUAL_TEMPORAL_TOPOLOGY"
    );
}

#[test]
fn corrupted_result_type_and_input_limit_fail_closed() {
    let mut result_type = compile(
        "x + 1.0",
        &[("x", parameter("bad_result", TemporalType::Scalar))],
    );
    let instruction = result_type.program.core_mut().blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::Arithmetic { .. }))
        .unwrap();
    instruction.type_id = CoreTypeId::new(u32::MAX);
    assert_eq!(
        failure(&result_type, &no_bindings(), "unsupported_type"),
        "RESIDUAL_TYPE_UNSUPPORTED"
    );

    let expression = compile(
        "input",
        &[("input", parameter("limit_template", TemporalType::Scalar))],
    );
    let template = expression.core().inputs()[0].clone();
    let request = request("input_limit");
    let execution = crate::program::expression::ExecutionBudget::default();
    let mut builder = super::super::builder::Builder::new(&request, execution.residual_ledger());
    for index in 0..=veac_ir::MAX_TEMPORAL_INPUTS {
        let mut input = template.clone();
        input.id = InputId::new(index as u32);
        let identity = CoreTemporalInputIdentity::Parameter {
            parameter_id: TemporalParameterId::new(format!("tpm_limit_{index}")).unwrap(),
            value_type: TemporalType::Scalar,
        };
        input.identity = CoreInputIdentity::Temporal(identity.clone());
        let result = builder.input(&input, &identity);
        if index < veac_ir::MAX_TEMPORAL_INPUTS {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err().code(), "RESIDUAL_INPUT_LIMIT");
        }
    }
}

fn failure(
    expression: &crate::program::expression::CompiledExpression,
    bindings: &ResidualBuildBindings,
    name: &str,
) -> &'static str {
    super::super::residualize_expression(expression, bindings, request(name))
        .unwrap_err()
        .code()
}

fn branch_condition(expression: &crate::program::expression::CompiledExpression) -> ValueId {
    let CoreTerminator::Branch { condition, .. } = expression.core().blocks()[0].terminator()
    else {
        panic!("fixture must branch");
    };
    *condition
}

fn branch_targets(expression: &crate::program::expression::CompiledExpression) -> [BlockId; 2] {
    let CoreTerminator::Branch {
        then_target,
        else_target,
        ..
    } = expression.core().blocks()[0].terminator()
    else {
        panic!("fixture must branch");
    };
    [*then_target, *else_target]
}
