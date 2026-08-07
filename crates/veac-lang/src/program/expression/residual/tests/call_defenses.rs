use super::support::*;
use crate::program::expression::{
    BuiltinFunction, CompiledExpression, CoreCallTarget, CoreInstructionKind, Value, ValueId,
};

#[test]
fn residual_extremum_and_clamp_enforce_verified_arity() {
    let input = [(
        "clock",
        parameter("arity_clock", veac_ir::TemporalType::Scalar),
    )];
    for (name, source, function) in [
        ("min_arity", "min(clock, 0.0)", BuiltinFunction::Min),
        (
            "clamp_arity",
            "clamp(clock, 0.0, 1.0)",
            BuiltinFunction::Clamp,
        ),
    ] {
        let mut expression = compile(source, &input);
        let CoreInstructionKind::Call { arguments, .. } =
            &mut builtin_mut(&mut expression, function).kind
        else {
            unreachable!()
        };
        arguments.pop();
        let error = failure(&expression, name);
        assert_eq!(error.code(), "RESIDUAL_CORE_CONTRACT");
        assert!(error.message().contains("arity mismatch"));
    }
}

#[test]
fn residual_ranges_reject_missing_steps_and_zero_steps() {
    let mut missing = compile("0 .. 4 by 2", &[]);
    let CoreInstructionKind::Range { step, .. } = &mut range_mut(&mut missing).kind else {
        unreachable!()
    };
    *step = Some(ValueId::new(u32::MAX));
    assert_eq!(
        failure(&missing, "missing_range_step").code(),
        "RESIDUAL_CORE_CONTRACT"
    );

    let mut zero = compile("0 .. 4 by 2", &[]);
    let step = match range(&zero).kind() {
        CoreInstructionKind::Range {
            step: Some(step), ..
        } => *step,
        _ => unreachable!(),
    };
    instruction_mut(&mut zero, step).kind = CoreInstructionKind::Literal(Value::Integer(0));
    assert_eq!(failure(&zero, "zero_range_step").code(), "VALUE_RANGE_STEP");
}

fn builtin_mut(
    expression: &mut CompiledExpression,
    function: BuiltinFunction,
) -> &mut crate::program::expression::CoreInstruction {
    expression
        .program
        .core_mut()
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| {
            matches!(
                value.kind,
                CoreInstructionKind::Call {
                    target: CoreCallTarget::Builtin(target),
                    ..
                } if target == function
            )
        })
        .unwrap()
}

fn range(expression: &CompiledExpression) -> &crate::program::expression::CoreInstruction {
    expression
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|value| matches!(value.kind(), CoreInstructionKind::Range { .. }))
        .unwrap()
}

fn range_mut(
    expression: &mut CompiledExpression,
) -> &mut crate::program::expression::CoreInstruction {
    expression
        .program
        .core_mut()
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| matches!(value.kind, CoreInstructionKind::Range { .. }))
        .unwrap()
}

fn instruction_mut(
    expression: &mut CompiledExpression,
    id: ValueId,
) -> &mut crate::program::expression::CoreInstruction {
    expression
        .program
        .core_mut()
        .blocks
        .iter_mut()
        .flat_map(|block| &mut block.instructions)
        .find(|value| value.id == id)
        .unwrap()
}

fn failure(
    expression: &CompiledExpression,
    name: &str,
) -> crate::program::expression::ResidualizationError {
    super::super::residualize_expression(expression, &no_bindings(), request(name)).unwrap_err()
}
