use super::super::slot::RuntimeValue;
use super::super::Evaluator;
use crate::program::expression::{
    compile_expression, CoreForEach, CoreInstructionKind, CoreTerminator, Environment,
    ExecutionBudget, ExpressionContext, TypeEnvironment, Value,
};
use crate::program::DomainOperationRegistry;

fn compiled(source: &str) -> crate::program::expression::CompiledExpression {
    compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap()
}

fn loop_value(expression: &crate::program::expression::CompiledExpression) -> &CoreForEach {
    expression
        .core()
        .blocks()
        .iter()
        .find_map(|block| match block.terminator() {
            CoreTerminator::ForEach(value) => Some(value.as_ref()),
            _ => None,
        })
        .unwrap()
}

fn evaluator<'a>(
    expression: &crate::program::expression::CompiledExpression,
    environment: &'a Environment,
    execution: &'a ExecutionBudget,
    domain: &'a DomainOperationRegistry,
) -> Evaluator<'a> {
    Evaluator::new(environment, execution, expression.registry_arc(), domain)
}

fn loop_slots(
    expression: &crate::program::expression::CompiledExpression,
    iterable: Value,
) -> Vec<Option<RuntimeValue>> {
    let mut values = std::iter::repeat_with(|| None)
        .take(expression.core().value_count())
        .collect::<Vec<_>>();
    values[loop_value(expression).iterable().index().unwrap()] =
        Some(RuntimeValue::Public(iterable));
    values
}

#[test]
fn for_each_runtime_rejects_non_iterable_and_forged_static_bound() {
    let expression = compiled("for value in [1] { value }");
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let mut evaluator = evaluator(&expression, &environment, &execution, &domain);
    let mut values = loop_slots(&expression, Value::Integer(1));
    let error = evaluator
        .for_each(
            loop_value(&expression),
            expression.verified(),
            &mut values,
            0,
        )
        .err()
        .unwrap();
    assert!(error.message().contains("not a verified iterable"));

    let list = Value::list(
        crate::program::expression::PrimitiveType::Integer.into(),
        vec![Value::Integer(1), Value::Integer(2)],
    )
    .unwrap();
    let mut values = loop_slots(&expression, list);
    let mut forged = loop_value(&expression).clone();
    forged.maximum_count = 1;
    let error = evaluator
        .for_each(&forged, expression.verified(), &mut values, 0)
        .err()
        .unwrap();
    assert!(error.message().contains("verified static bound"));
}

#[test]
fn for_each_runtime_rejects_missing_body_after_verified_load() {
    let expression = compiled("for value in [1] { value }");
    let iterable = expression
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find_map(|instruction| match instruction.kind() {
            CoreInstructionKind::List { .. } => Some(
                Value::list(
                    crate::program::expression::PrimitiveType::Integer.into(),
                    vec![Value::Integer(1)],
                )
                .unwrap(),
            ),
            _ => None,
        })
        .unwrap();
    let mut forged = loop_value(&expression).clone();
    forged.body = crate::program::expression::ClosureDefinitionId::new(99);
    let environment = Environment::new();
    let execution = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let mut evaluator = evaluator(&expression, &environment, &execution, &domain);
    let mut values = loop_slots(&expression, iterable);
    let error = evaluator
        .for_each(&forged, expression.verified(), &mut values, 0)
        .err()
        .unwrap();
    assert!(error.message().contains("body is unavailable"));
}
