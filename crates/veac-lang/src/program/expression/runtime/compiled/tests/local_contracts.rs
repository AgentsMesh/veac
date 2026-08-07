use super::super::slot::RuntimeValue;
use super::super::Evaluator;
use crate::program::expression::{
    compile_expression, CoreInstructionKind, Environment, ExecutionBudget, ExpressionContext,
    LocalSlotId, TypeEnvironment, Value,
};
use crate::program::DomainOperationRegistry;

#[test]
fn local_runtime_rejects_uninitialized_duplicate_and_unknown_access() {
    let compiled = compile_expression(
        "{ var value = 1; value }",
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap();
    let instructions = compiled.core().blocks()[0].instructions();
    let initializer = instructions
        .iter()
        .find(|value| matches!(value.kind(), CoreInstructionKind::LocalInit { .. }))
        .unwrap();
    let getter = instructions
        .iter()
        .find(|value| matches!(value.kind(), CoreInstructionKind::LocalGet { .. }))
        .unwrap();
    let CoreInstructionKind::LocalInit { value, .. } = initializer.kind() else {
        unreachable!()
    };
    let mut values = std::iter::repeat_with(|| None)
        .take(compiled.core().value_count())
        .collect::<Vec<_>>();
    values[value.index().unwrap()] = Some(RuntimeValue::Public(Value::Integer(1)));
    let mut locals = vec![None];
    let environment = Environment::new();
    let budget = ExecutionBudget::default();
    let domain = DomainOperationRegistry::standard();
    let evaluator = Evaluator::new(&environment, &budget, compiled.registry_arc(), &domain);

    assert_eq!(
        evaluator
            .local(getter, &values, &mut locals)
            .unwrap_err()
            .code(),
        "EXPRESSION_RUNTIME_CONTRACT"
    );
    assert_eq!(
        evaluator.local(initializer, &values, &mut locals).unwrap(),
        Value::Integer(1)
    );
    assert_eq!(
        evaluator
            .local(initializer, &values, &mut locals)
            .unwrap_err()
            .code(),
        "EXPRESSION_RUNTIME_CONTRACT"
    );

    let mut unknown = getter.clone();
    unknown.kind = CoreInstructionKind::LocalGet {
        slot: LocalSlotId::new(99),
    };
    assert_eq!(
        evaluator
            .local(&unknown, &values, &mut locals)
            .unwrap_err()
            .code(),
        "EXPRESSION_RUNTIME_CONTRACT"
    );
}
