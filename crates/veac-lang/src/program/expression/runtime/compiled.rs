use std::sync::Arc;

use super::super::core::{CompiledExpression, FunctionRegistry};
use super::super::{
    DomainOrigin, ExecutionBudget, ExecutionDefinition, ExecutionFrame, ExpressionError,
    ExpressionLoopFrame, Value, ValueLookup,
};
use super::domain_graph::{DomainGraphTransaction, FrozenDomainGraph};
use crate::program::{DomainOperationRegistry, DomainType};

mod aggregate;
mod call;
mod closure;
mod collection;
mod control;
mod domain;
mod entry;
mod for_each;
mod input;
mod instruction;
mod iterable;
mod local;
mod nominal;
mod program;
mod range;
mod slot;
mod temporal_attachment;
mod value;

use value::program_span;

pub(in crate::program::expression) fn execute(
    expression: &CompiledExpression,
    environment: &dyn ValueLookup,
    execution: &ExecutionBudget,
) -> Result<Value, ExpressionError> {
    if expression
        .result_type()
        .contains_domain_in(expression.verified().nominal_types())
        != Some(false)
    {
        return Err(ExpressionError::new(
            "EXPRESSION_DOMAIN_RESULT_BOUNDARY",
            "domain values require the project execution boundary",
            program_span(expression.core()),
        ));
    }
    let domain_registry = DomainOperationRegistry::shared();
    validate_domain_identity(expression, domain_registry)?;
    Evaluator::new(
        environment,
        execution,
        expression.registry_arc(),
        domain_registry,
    )
    .program(expression.verified(), &[], &[], 0, None, None)
}

#[allow(dead_code)]
pub(in crate::program) fn execute_project(
    expression: &CompiledExpression,
    environment: &dyn ValueLookup,
    execution: &ExecutionBudget,
) -> Result<FrozenDomainGraph, ExpressionError> {
    if expression.result_type().as_domain() != Some(DomainType::Project) {
        return Err(ExpressionError::new(
            "EXPRESSION_PROJECT_RESULT",
            "project execution requires a Project result",
            program_span(expression.core()),
        ));
    }
    let domain_registry = DomainOperationRegistry::shared();
    validate_domain_identity(expression, domain_registry)?;
    let mut evaluator = Evaluator::new(
        environment,
        execution,
        expression.registry_arc(),
        domain_registry,
    );
    let value = evaluator.program(expression.verified(), &[], &[], 0, None, None)?;
    evaluator
        .domain
        .freeze(&value, program_span(expression.core()))
}

pub(in crate::program) use entry::execute as execute_entry;

pub(in crate::program) fn execute_value_entry(
    entry: &super::super::CompiledFunction,
    functions: &super::super::FunctionMap,
    types: &crate::program::TypeRegistry,
    arguments: &[Value],
    environment: &dyn super::super::ValueLookup,
    execution: &ExecutionBudget,
) -> Result<Value, ExpressionError> {
    if entry.return_type().contains_domain_in(types) != Some(false) {
        return Err(ExpressionError::new(
            "EXPRESSION_DOMAIN_RESULT_BOUNDARY",
            "host entry cannot return domain values",
            program_span(entry.body()),
        ));
    }
    let registry = DomainOperationRegistry::shared();
    validate_core_domain_identity(entry.body(), registry)?;
    let mut evaluator = Evaluator::new(environment, execution, functions.registry_arc(), registry);
    let frame = ExecutionDefinition::function(entry).map(|value| ExecutionFrame::new(value, None));
    evaluator.program(entry.verified_body(), arguments, &[], 0, Some(entry), frame)
}

struct Evaluator<'a> {
    environment: &'a dyn ValueLookup,
    execution: &'a ExecutionBudget,
    registry: Arc<FunctionRegistry>,
    domain: DomainGraphTransaction<'a>,
    frames: Vec<ExecutionFrame>,
    iterations: Vec<ExpressionLoopFrame>,
}

impl<'a> Evaluator<'a> {
    fn new(
        environment: &'a dyn ValueLookup,
        execution: &'a ExecutionBudget,
        registry: Arc<FunctionRegistry>,
        domain_registry: &'a DomainOperationRegistry,
    ) -> Self {
        Self {
            environment,
            execution,
            registry,
            domain: DomainGraphTransaction::new(domain_registry, execution),
            frames: Vec::new(),
            iterations: Vec::new(),
        }
    }

    fn enter(&mut self, span: std::ops::Range<usize>) -> Result<(), ExpressionError> {
        self.execution.charge_node(span)
    }

    fn origin(&self, span: std::ops::Range<usize>) -> Option<DomainOrigin> {
        DomainOrigin::capture(&self.frames, &self.iterations, span)
    }
}

#[cfg(test)]
#[path = "compiled/tests.rs"]
mod tests;

fn validate_domain_identity(
    expression: &CompiledExpression,
    registry: &DomainOperationRegistry,
) -> Result<(), ExpressionError> {
    validate_core_domain_identity(expression.core(), registry)
}

fn validate_core_domain_identity(
    core: &super::super::CoreProgram,
    registry: &DomainOperationRegistry,
) -> Result<(), ExpressionError> {
    (core.domain_opset() == registry.version()
        && core.domain_registry_digest() == registry.digest())
    .then_some(())
    .ok_or_else(|| {
        ExpressionError::new(
            "EXPRESSION_RUNTIME_CONTRACT",
            "verified Core domain operation identity does not match runtime",
            program_span(core),
        )
    })
}
