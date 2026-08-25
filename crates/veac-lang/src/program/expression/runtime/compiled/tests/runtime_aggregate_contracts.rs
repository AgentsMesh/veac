use super::super::slot::RuntimeValue;
use super::super::Evaluator;
use crate::program::expression::{
    compile_expression, CoreInstruction, CoreInstructionKind, CoreTypeId, Environment,
    ExecutionBudget, ExpressionContext, TypeEnvironment, Value,
};
use crate::program::DomainOperationRegistry;

fn compiled(source: &str) -> crate::program::expression::CompiledExpression {
    compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap()
}

fn aggregate(expression: &crate::program::expression::CompiledExpression) -> CoreInstruction {
    expression
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|value| matches!(value.kind(), CoreInstructionKind::Collection { .. }))
        .unwrap()
        .clone()
}

fn evaluator<'a>(
    expression: &crate::program::expression::CompiledExpression,
    environment: &'a Environment,
    execution: &'a ExecutionBudget,
    domain: &'a DomainOperationRegistry,
) -> Evaluator<'a> {
    Evaluator::new(environment, execution, expression.registry_arc(), domain)
}

fn inputs(
    expression: &crate::program::expression::CompiledExpression,
    instruction: &CoreInstruction,
    evaluator: &Evaluator<'_>,
) -> Vec<Option<RuntimeValue>> {
    let CoreInstructionKind::Collection {
        iterable, callable, ..
    } = instruction.kind()
    else {
        unreachable!()
    };
    let closure = expression
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find(|value| matches!(value.kind(), CoreInstructionKind::Closure { .. }))
        .unwrap();
    let CoreInstructionKind::Closure {
        definition,
        captures,
    } = closure.kind()
    else {
        unreachable!()
    };
    let closure_value = evaluator
        .closure(*definition, captures, closure, expression.verified(), &[])
        .unwrap();
    let list = Value::list(
        crate::program::expression::PrimitiveType::Integer.into(),
        vec![Value::Integer(1)],
    )
    .unwrap();
    let count = usize::max(iterable.value() as usize, callable.value() as usize) + 1;
    let mut values = std::iter::repeat_with(|| None)
        .take(count)
        .collect::<Vec<_>>();
    values[iterable.index().unwrap()] = Some(RuntimeValue::Public(list));
    values[callable.index().unwrap()] = Some(RuntimeValue::Public(closure_value));
    values
}

#[test]
fn aggregate_runtime_rejects_unknown_and_non_list_result_types() {
    let expression = compiled("map([1], fn(value: int) -> int effect pure { value })");
    let base = aggregate(&expression);
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let mut evaluator = evaluator(&expression, &environment, &execution, &domain);
    let values = inputs(&expression, &base, &evaluator);

    let mut missing = base.clone();
    missing.type_id = CoreTypeId::new(u32::MAX);
    assert!(evaluator
        .aggregate_instruction(&missing, expression.verified(), &values, 0)
        .unwrap_err()
        .message()
        .contains("result type is unavailable"));

    let mut scalar = base;
    scalar.type_id = expression.core().blocks()[0].instructions()[0].type_id();
    assert!(evaluator
        .aggregate_instruction(&scalar, expression.verified(), &values, 0)
        .unwrap_err()
        .message()
        .contains("not a verified list"));
}
