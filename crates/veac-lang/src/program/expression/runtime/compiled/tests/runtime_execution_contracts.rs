use super::super::slot::RuntimeValue;
use super::super::Evaluator;
use crate::program::expression::{
    compile_expression, CoreInstruction, CoreInstructionKind, Environment, ExecutionBudget,
    ExpressionContext, TypeEnvironment, Value, ValueId, MAX_FUNCTION_CALL_DEPTH,
};
use crate::program::{DomainOperationRegistry, FieldIndex};

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

fn new_evaluator<'a>(
    environment: &'a Environment,
    execution: &'a ExecutionBudget,
    expression: &crate::program::expression::CompiledExpression,
    domain: &'a DomainOperationRegistry,
) -> Evaluator<'a> {
    Evaluator::new(environment, execution, expression.registry_arc(), domain)
}

#[test]
fn aggregate_runtime_rejects_non_iterable_and_missing_initial() {
    let expression = compiled("map([1], fn(value: int) -> int effect pure { value })");
    let aggregate = instruction(expression.core(), |kind| {
        matches!(kind, CoreInstructionKind::Collection { .. })
    });
    let CoreInstructionKind::Collection {
        iterable, callable, ..
    } = aggregate.kind
    else {
        unreachable!()
    };
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let mut evaluator = new_evaluator(&environment, &execution, &expression, &domain);
    let values = slots([
        (iterable, RuntimeValue::Public(Value::Integer(1))),
        (callable, RuntimeValue::Public(Value::Integer(2))),
    ]);
    assert!(evaluator
        .aggregate_instruction(&aggregate, expression.verified(), &values, 0)
        .unwrap_err()
        .message()
        .contains("not a verified iterable"));

    let fold =
        compiled("fold([1], 0, fn(total: int, value: int) -> int effect pure { total + value })");
    let mut aggregate = instruction(fold.core(), |kind| {
        matches!(kind, CoreInstructionKind::Collection { .. })
    });
    let CoreInstructionKind::Collection { initial, .. } = &mut aggregate.kind else {
        unreachable!()
    };
    *initial = None;
    let CoreInstructionKind::Collection {
        iterable, callable, ..
    } = aggregate.kind
    else {
        unreachable!()
    };
    let mut evaluator = new_evaluator(&environment, &execution, &fold, &domain);
    let closure = instruction(fold.core(), |kind| {
        matches!(kind, CoreInstructionKind::Closure { .. })
    });
    let CoreInstructionKind::Closure {
        definition,
        ref captures,
    } = closure.kind
    else {
        unreachable!()
    };
    let callable_value = evaluator
        .closure(definition, captures, &closure, fold.verified(), &[])
        .unwrap();
    let iterable_value = Value::list(
        crate::program::expression::PrimitiveType::Integer.into(),
        vec![Value::Integer(1)],
    )
    .unwrap();
    let values = slots([
        (iterable, RuntimeValue::Public(iterable_value)),
        (callable, RuntimeValue::Public(callable_value)),
    ]);
    assert!(evaluator
        .aggregate_instruction(&aggregate, fold.verified(), &values, 0)
        .unwrap_err()
        .message()
        .contains("initial value is unavailable"));
}

#[test]
fn nominal_runtime_rejects_wrong_projection_receiver_and_field() {
    let expression = compiled("1");
    let instruction = instruction(expression.core(), |_| true);
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let evaluator = new_evaluator(&environment, &execution, &expression, &domain);
    let values = slots([(ValueId::new(0), RuntimeValue::Public(Value::Integer(1)))]);
    assert!(evaluator
        .struct_project(ValueId::new(0), FieldIndex::new(0), &instruction, &values)
        .unwrap_err()
        .message()
        .contains("receiver is invalid"));
}

#[test]
fn for_each_runtime_enforces_call_depth_before_iteration() {
    let expression = compiled("for value in [1] { value }");
    let loop_value = expression
        .core()
        .blocks()
        .iter()
        .find_map(|block| match block.terminator() {
            crate::program::expression::CoreTerminator::ForEach(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let mut evaluator = new_evaluator(&environment, &execution, &expression, &domain);
    let mut values = std::iter::repeat_with(|| None)
        .take(expression.core().value_count())
        .collect::<Vec<_>>();
    let error = evaluator
        .for_each(
            loop_value,
            expression.verified(),
            &mut values,
            MAX_FUNCTION_CALL_DEPTH,
        )
        .err()
        .expect("call-depth limit must reject the loop");
    assert_eq!(error.code(), "EXPRESSION_CALL_DEPTH_LIMIT");
}
