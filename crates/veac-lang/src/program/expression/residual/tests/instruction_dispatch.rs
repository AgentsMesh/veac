use super::support::*;
use crate::program::expression::{
    CoreInstructionKind, CoreTemporalInputIdentity, PrimitiveType, TypeEnvironment, Value,
};
use veac_ir::{TemporalParameterId, TemporalType};

#[test]
fn residual_dispatch_rejects_missing_build_input_at_instruction_boundary() {
    let types = TypeEnvironment::from([("value".to_owned(), PrimitiveType::Integer.into())]);
    let expression = compile_with_build("value + 1", types, &[]);
    let error = super::super::residualize_expression(
        &expression,
        &no_bindings(),
        request("missing_build_dispatch"),
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_BUILD_INPUT_MISSING");
}

#[test]
fn residual_dispatch_executes_capture_instruction_in_collection_callback() {
    let expression = compile(
        "{ let offset = 3; map([1, 2], fn(value: int) -> int effect pure { value + offset }) }",
        &[],
    );
    let result = residualize(&expression, &no_bindings(), "capture_dispatch");
    let crate::program::expression::ResidualRuntimeValue::Concrete(value) = result.value() else {
        panic!("collection result must remain concrete")
    };
    assert_eq!(value.render(), "[4, 5]");
}

#[test]
fn unsupported_local_instruction_fails_closed_before_residual_execution() {
    let types = TypeEnvironment::from([("value".to_owned(), PrimitiveType::Integer.into())]);
    let mut expression = compile_with_build("value + 1", types, &[]);
    let instruction = expression.program.core_mut().blocks[0]
        .instructions
        .iter_mut()
        .find(|value| matches!(value.kind, CoreInstructionKind::Arithmetic { .. }))
        .unwrap();
    instruction.kind = CoreInstructionKind::LocalGet {
        slot: crate::program::expression::LocalSlotId::new(0),
    };
    let error = super::super::residualize_expression(
        &expression,
        &binding("value", Value::Integer(1)),
        request("unsupported_local"),
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_INSTRUCTION_UNSUPPORTED");
}

#[test]
fn temporal_input_dispatch_covers_parameter_identity() {
    let identity = CoreTemporalInputIdentity::Parameter {
        parameter_id: TemporalParameterId::new("tpm_instruction_dispatch").unwrap(),
        value_type: TemporalType::Integer,
    };
    let expression = compile("value + 1", &[("value", identity)]);
    let result = residualize(&expression, &no_bindings(), "temporal_input_dispatch");
    assert_eq!(result.inputs().len(), 1);
}
