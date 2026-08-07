use std::sync::Arc;

use crate::program::expression::runtime;
use crate::program::expression::{
    compile_expression, compile_functions, ClosureDefinitionId, CoreCallTarget,
    CoreInstructionKind, Environment, ExecutionBudget, ExpressionContext, FunctionDefinition,
    PrimitiveType, TypeEnvironment, ValueType,
};
use crate::program::{DomainOpsetVersion, DomainRegistryDigest};

#[test]
fn runtime_rechecks_direct_call_domain_identity() {
    let helper = FunctionDefinition::new(
        "helper",
        Vec::new(),
        ValueType::primitive(PrimitiveType::Integer),
        "{ 1 }",
    );
    let functions = compile_functions(&ExpressionContext::empty(), &[helper]).unwrap();
    let mut expression =
        compile_expression("helper()", &TypeEnvironment::new(), &functions).unwrap();
    let target = expression
        .core()
        .blocks()
        .iter()
        .flat_map(|block| block.instructions())
        .find_map(|instruction| match instruction.kind() {
            CoreInstructionKind::Call {
                target: CoreCallTarget::User(id),
                ..
            } => Some(*id),
            _ => None,
        })
        .unwrap();
    let registry = Arc::make_mut(&mut expression.registry);
    let function = Arc::make_mut(registry.get_mut(target).unwrap());
    function.body.core_mut().domain_opset = DomainOpsetVersion::from_raw(99);

    assert_runtime_contract(expression);
}

#[test]
fn runtime_rechecks_invoked_closure_domain_identity() {
    let source = "{ let callback = fn() -> int effect pure { 1 }; callback() }";
    let mut expression =
        compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap();
    expression
        .program
        .closure_core_mut(ClosureDefinitionId::new(0))
        .unwrap()
        .domain_registry_digest = DomainRegistryDigest::from_bytes([8; 32]);

    assert_runtime_contract(expression);
}

fn assert_runtime_contract(expression: super::super::CompiledExpression) {
    let error = runtime::execute(
        &expression,
        &Environment::new(),
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_RUNTIME_CONTRACT");
}
