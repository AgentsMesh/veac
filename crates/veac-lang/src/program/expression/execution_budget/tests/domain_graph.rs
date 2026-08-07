use std::sync::Arc;

use crate::program::{DomainOperationId, DomainOperationRegistry, DomainType};

use super::super::super::runtime::domain_graph::DomainGraphTransaction;
use super::super::super::Value;
use super::super::super::{
    compile_expression, runtime, Environment, ExpressionContext, TypeEnvironment,
};
use super::{budget, set_limit, unlimited, used, ExecutionBudget, Resource};

#[path = "domain_graph/refinement.rs"]
mod refinement;
#[path = "domain_graph/relation.rs"]
mod relation;
#[path = "domain_graph/resource.rs"]
mod resource;
#[path = "domain_graph/support.rs"]
mod support;
#[path = "domain_graph/update.rs"]
mod update;
use support::*;

#[test]
fn project_runtime_entity_budget_accepts_exact_and_never_publishes_one_short() {
    let expression = compile_expression(
        "{ let timeline = sequence(identifier(\"main\"), \"Main\", \
         sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)); \
         project(identifier(\"project\"), project_settings(600)) \
           .with_sequence(timeline).entry(timeline) }",
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap();
    let exact = budget(Resource::EmittedEntities, 2);
    let frozen = runtime::execute_project(&expression, &Environment::new(), &exact).unwrap();
    assert_eq!(frozen.entity_count(), 2);
    assert_eq!(used(&exact, Resource::EmittedEntities), 2);

    let short = budget(Resource::EmittedEntities, 1);
    let error = runtime::execute_project(&expression, &Environment::new(), &short).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(used(&short, Resource::EmittedEntities), 1);
}

#[test]
fn description_byte_budget_accepts_exact_and_rejects_one_short() {
    let registry = DomainOperationRegistry::standard();
    let measured = budget(Resource::EmittedBytes, usize::MAX);
    let mut graph = DomainGraphTransaction::new(&registry, &measured);
    canvas(&mut graph);
    let bytes = used(&measured, Resource::EmittedBytes);

    let exact = budget(Resource::EmittedBytes, bytes);
    let mut graph = DomainGraphTransaction::new(&registry, &exact);
    canvas(&mut graph);
    assert_eq!(graph.record_count(), 1);
    assert_eq!(used(&exact, Resource::EmittedBytes), bytes);

    let short = budget(Resource::EmittedBytes, bytes - 1);
    let mut graph = DomainGraphTransaction::new(&registry, &short);
    let error = evaluate_result(
        &mut graph,
        DomainOperationId::Canvas,
        vec![length(1080), length(1920)],
    )
    .unwrap_err();
    assert_atomic_failure(&graph, &short, error, 0, 0, 0);
}

#[test]
fn entity_budget_accepts_exact_and_rejects_without_byte_charge() {
    let registry = DomainOperationRegistry::standard();
    let exact = budget(Resource::EmittedEntities, 1);
    let mut graph = DomainGraphTransaction::new(&registry, &exact);
    sequence(&mut graph, "main");
    assert_eq!(used(&exact, Resource::EmittedEntities), 1);

    let short = budget(Resource::EmittedEntities, 0);
    let mut graph = DomainGraphTransaction::new(&registry, &short);
    let operands = sequence_operands(&mut graph, "main");
    let before = snapshot(&graph, &short);
    let error = evaluate_result(&mut graph, DomainOperationId::Sequence, operands).unwrap_err();
    assert_atomic_failure(
        &graph,
        &short,
        error,
        before.records,
        before.bytes,
        before.entities,
    );
}

#[test]
fn multi_axis_reservation_is_atomic_before_arena_mutation() {
    let registry = DomainOperationRegistry::standard();
    let measured = ExecutionBudget::with_resource_limits(unlimited());
    let mut graph = DomainGraphTransaction::new(&registry, &measured);
    sequence(&mut graph, "main");
    let total = used(&measured, Resource::EmittedBytes);

    let mut limits = unlimited();
    set_limit(&mut limits, Resource::EmittedEntities, 1);
    set_limit(&mut limits, Resource::EmittedBytes, total - 1);
    let short = ExecutionBudget::with_resource_limits(limits);
    let mut graph = DomainGraphTransaction::new(&registry, &short);
    let operands = sequence_operands(&mut graph, "main");
    let before = snapshot(&graph, &short);
    let error = evaluate_result(&mut graph, DomainOperationId::Sequence, operands).unwrap_err();
    assert_atomic_failure(
        &graph,
        &short,
        error,
        before.records,
        before.bytes,
        before.entities,
    );
}

fn identifier(value: &str) -> Value {
    Value::Identifier(Arc::from(value))
}
