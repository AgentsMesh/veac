use super::super::slot::RuntimeValue;
use super::super::Evaluator;
use crate::program::expression::core::{
    ClosureDefinitionId, CoreCallTarget, CoreInstruction, CoreInstructionKind, CoreTypeId,
    FunctionId, InputId, ValueId,
};
use crate::program::expression::{
    compile_expression, Environment, ExecutionBudget, ExpressionContext, TypeEnvironment, Value,
    MAX_FUNCTION_CALL_DEPTH,
};
use crate::program::DomainOperationRegistry;

#[path = "runtime_contracts/map.rs"]
mod map;

fn compiled(source: &str) -> crate::program::expression::CompiledExpression {
    compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap()
}

fn instruction(
    program: &crate::program::expression::CoreProgram,
    predicate: impl Fn(&CoreInstructionKind) -> bool,
) -> CoreInstruction {
    program
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|value| predicate(value.kind()))
        .unwrap()
        .clone()
}

fn slots(values: impl IntoIterator<Item = (ValueId, RuntimeValue)>) -> Vec<Option<RuntimeValue>> {
    let values = values.into_iter().collect::<Vec<_>>();
    let count = values
        .iter()
        .map(|(id, _)| id.value() as usize + 1)
        .max()
        .unwrap_or(0);
    let mut output = std::iter::repeat_with(|| None)
        .take(count)
        .collect::<Vec<_>>();
    for (id, value) in values {
        output[id.value() as usize] = Some(value);
    }
    output
}

#[test]
fn sequence_and_range_runtime_contracts_fail_closed() {
    let list = compiled("[1]");
    let list_instruction = instruction(list.core(), |kind| {
        matches!(kind, CoreInstructionKind::List { .. })
    });
    let CoreInstructionKind::List { elements } = &list_instruction.kind else {
        unreachable!()
    };
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let evaluator = Evaluator::new(&environment, &execution, list.registry_arc(), &domain);
    let bool_values = slots([(elements[0], RuntimeValue::Public(Value::Bool(true)))]);
    assert!(evaluator
        .sequence(
            elements,
            false,
            &list_instruction,
            list.core(),
            &bool_values
        )
        .is_err());
    let mut unavailable = list_instruction.clone();
    unavailable.type_id = CoreTypeId::new(u32::MAX);
    assert!(evaluator
        .sequence(elements, false, &unavailable, list.core(), &bool_values)
        .is_err());
    let integer = instruction(list.core(), |kind| {
        matches!(kind, CoreInstructionKind::Literal(Value::Integer(_)))
    });
    assert!(evaluator
        .sequence(elements, false, &integer, list.core(), &bool_values)
        .is_err());

    let range = compiled("1 .. 4 by 1");
    let range_instruction = instruction(range.core(), |kind| {
        matches!(kind, CoreInstructionKind::Range { .. })
    });
    let CoreInstructionKind::Range { start, end, step } = range_instruction.kind else {
        unreachable!()
    };
    let evaluator = Evaluator::new(&environment, &execution, range.registry_arc(), &domain);
    let values = slots([
        (start, RuntimeValue::Public(Value::Bool(false))),
        (end, RuntimeValue::Public(Value::Integer(4))),
    ]);
    assert!(evaluator
        .range(start, end, step, &range_instruction, range.core(), &values)
        .is_err());
    let zero = slots([
        (start, RuntimeValue::Public(Value::Integer(1))),
        (end, RuntimeValue::Public(Value::Integer(4))),
        (step.unwrap(), RuntimeValue::Public(Value::Integer(0))),
    ]);
    assert_eq!(
        evaluator
            .range(start, end, step, &range_instruction, range.core(), &zero)
            .err()
            .unwrap()
            .code(),
        "EXPRESSION_RANGE_STEP"
    );
}

#[test]
fn call_and_public_instruction_contracts_reject_unverified_operands() {
    let literal = compiled("1");
    let mut instruction = instruction(literal.core(), |_| true);
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let mut evaluator = Evaluator::new(&environment, &execution, literal.registry_arc(), &domain);
    for kind in [
        CoreInstructionKind::Input(InputId::new(99)),
        CoreInstructionKind::Parameter(99),
        CoreInstructionKind::Capture(99),
    ] {
        instruction.kind = kind;
        let mut values = std::iter::once(None).collect::<Vec<_>>();
        let mut locals = Vec::new();
        assert!(evaluator
            .instruction(
                &instruction,
                literal.verified(),
                &mut values,
                &mut locals,
                &[],
                &[],
                &[],
                0,
                None,
            )
            .is_err());
    }
    assert!(evaluator
        .call(
            CoreCallTarget::User(FunctionId::from_bytes([9; 32])),
            Vec::new(),
            &instruction,
            0,
            None,
        )
        .is_err());
    assert!(evaluator
        .call(
            CoreCallTarget::User(FunctionId::from_bytes([9; 32])),
            Vec::new(),
            &instruction,
            MAX_FUNCTION_CALL_DEPTH,
            None,
        )
        .is_err());
    assert!(evaluator
        .closure(
            ClosureDefinitionId::new(u32::MAX),
            &[],
            &instruction,
            literal.verified(),
            &[],
        )
        .is_err());
    assert!(evaluator
        .invoke(Value::Integer(1), Vec::new(), &instruction, 0)
        .is_err());
    assert!(evaluator
        .invoke(
            Value::Integer(1),
            Vec::new(),
            &instruction,
            MAX_FUNCTION_CALL_DEPTH,
        )
        .is_err());
}
