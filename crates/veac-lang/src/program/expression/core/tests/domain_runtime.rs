use super::domain_fixture::{compiled, project_program};
use crate::program::expression::runtime;
use crate::program::expression::{
    ArithmeticOperator, CoreInstructionKind, Effect, Environment, ExecutionBudget,
    ExpressionContext, FunctionDefinition, FunctionParameter, PrimitiveType, TypeEnvironment,
    ValueType,
};
use crate::program::{DomainOperationId, DomainOpsetVersion, DomainRegistryDigest, DomainType};

#[test]
fn verified_core_executes_into_a_frozen_connected_project() {
    let expression = compiled(project_program());
    let frozen = runtime::execute_project(
        &expression,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap();
    assert_eq!(frozen.root().domain_type(), DomainType::Project);
    assert_eq!(frozen.root_logical_key(), ["project"]);
    assert_eq!(frozen.entity_count(), 2);
    assert_eq!(frozen.record_count(), 6);
    assert_eq!(
        frozen.operation(frozen.root()),
        Some(DomainOperationId::ProjectEntry)
    );
}

#[test]
fn ordinary_expression_boundary_never_leaks_an_arena_handle() {
    let expression = compiled(project_program());
    let error = runtime::execute(
        &expression,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_DOMAIN_RESULT_BOUNDARY");
}

#[test]
fn project_boundary_requires_an_exact_project_result() {
    let expression = crate::program::expression::compile_expression(
        "1",
        &crate::program::expression::TypeEnvironment::new(),
        &crate::program::expression::ExpressionContext::empty(),
    )
    .unwrap();
    let error = runtime::execute_project(
        &expression,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_PROJECT_RESULT");
}

#[test]
fn closure_invocation_reuses_the_project_graph_transaction() {
    let source = r#"{
        let make_sequence = fn() -> Sequence effect emit {
            sequence(identifier("main"))
        };
        project(identifier("project"), canvas(1080px, 1920px), fps(30))
            .with_sequence(make_sequence())
            .entry(identifier("main"))
    }"#;
    let expression = crate::program::expression::compile_expression(
        source,
        &crate::program::expression::TypeEnvironment::new(),
        &crate::program::expression::ExpressionContext::empty(),
    )
    .unwrap();
    let frozen = runtime::execute_project(
        &expression,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap();
    assert_eq!(frozen.entity_count(), 2);
    assert_eq!(frozen.record_count(), 6);
    assert_eq!(frozen.root_logical_key(), ["project"]);
}

#[test]
fn runtime_rechecks_verified_domain_registry_identity() {
    let mut wrong_opset = compiled(project_program());
    wrong_opset.program.core_mut().domain_opset = DomainOpsetVersion::from_raw(99);
    assert_runtime_contract(wrong_opset);

    let mut wrong_digest = compiled(project_program());
    wrong_digest.program.core_mut().domain_registry_digest =
        DomainRegistryDigest::from_bytes([9; 32]);
    assert_runtime_contract(wrong_digest);
}

fn assert_runtime_contract(expression: super::super::CompiledExpression) {
    let error = runtime::execute_project(
        &expression,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_RUNTIME_CONTRACT");
}

#[test]
fn a_graph_emitting_call_failure_cannot_publish_a_partial_project() {
    let helper = FunctionDefinition::new(
        "broken_sequence",
        vec![FunctionParameter::new(
            "divisor",
            ValueType::primitive(PrimitiveType::Integer),
        )],
        ValueType::domain(DomainType::Sequence),
        "{ let emitted = sequence(identifier(\"main\")); \
         let ignored = 1 / divisor; emitted }",
    );
    let context =
        crate::program::expression::compile_functions(&ExpressionContext::empty(), &[helper])
            .unwrap();
    let helper = context.functions().lookup("broken_sequence").unwrap();
    assert_eq!(helper.summary().effect(), Effect::GraphEmit);
    let kinds = helper.body().blocks()[0].instructions();
    let emit = kinds
        .iter()
        .position(|value| matches!(value.kind(), CoreInstructionKind::GraphEmit { .. }))
        .unwrap();
    let failure = kinds
        .iter()
        .position(|value| {
            matches!(
                value.kind(),
                CoreInstructionKind::Arithmetic {
                    operator: ArithmeticOperator::Divide,
                    ..
                }
            )
        })
        .unwrap();
    assert!(emit < failure);

    let expression = crate::program::expression::compile_expression(
        "project(identifier(\"project\"), canvas(1080px, 1920px), fps(30))\
         .with_sequence(broken_sequence(0)).entry(identifier(\"main\"))",
        &TypeEnvironment::new(),
        &context,
    )
    .unwrap();
    let error = runtime::execute_project(
        &expression,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .expect_err("a failed transaction must not return a frozen graph");
    assert_eq!(error.code(), "EXPRESSION_DIVIDE_BY_ZERO");
}
