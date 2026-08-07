use super::super::super::{
    CompiledFunction, ExecutionBudget, ExecutionDefinition, ExecutionFrame, ExpressionError,
    FunctionMap, ProgramIdentity,
};
use super::super::domain_graph::FrozenDomainGraph;
use super::value::program_span;
use super::{validate_core_domain_identity, Evaluator};
use crate::program::{DomainOperationRegistry, DomainType};

pub(in crate::program) fn execute(
    entry: &CompiledFunction,
    functions: &FunctionMap,
    execution: &ExecutionBudget,
    identity: ProgramIdentity,
    environment: &dyn super::super::super::ValueLookup,
) -> Result<FrozenDomainGraph, ExpressionError> {
    if entry.return_type().as_domain() != Some(DomainType::Project) {
        return Err(ExpressionError::new(
            "EXPRESSION_PROJECT_RESULT",
            "executable entry requires a Project result",
            program_span(entry.body()),
        ));
    }
    let domain_registry = DomainOperationRegistry::shared();
    validate_core_domain_identity(entry.body(), domain_registry)?;
    let mut evaluator = Evaluator::new(
        environment,
        execution,
        functions.registry_arc(),
        domain_registry,
    );
    evaluator
        .domain
        .set_program_identity(identity, program_span(entry.body()))?;
    let context = evaluator.domain.host_context(program_span(entry.body()))?;
    let frame = ExecutionDefinition::function(entry).map(|value| ExecutionFrame::new(value, None));
    let value = evaluator.program(
        entry.verified_body(),
        &[context],
        &[],
        0,
        Some(entry),
        frame,
    )?;
    evaluator.domain.freeze(&value, program_span(entry.body()))
}
