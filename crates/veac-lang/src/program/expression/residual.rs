mod budget;
mod builder;
mod closure;
mod closure_state;
mod control;
mod convert;
mod domain;
mod effect;
mod error;
mod evaluator;
mod for_each;
mod input;
mod instruction;
mod map_state;
mod model;
mod temporal;

pub use error::ResidualizationError;
pub use model::{
    ResidualBuildBindings, ResidualInput, ResidualRuntimeValue, ResidualValue,
    ResidualizationLimits, ResidualizationRequest, ResidualizedExpression,
};

use super::{CompiledExpression, ExecutionBudget, ResidualLedger};

pub fn residualize_expression(
    expression: &CompiledExpression,
    build_inputs: &ResidualBuildBindings,
    request: ResidualizationRequest,
) -> Result<ResidualizedExpression, ResidualizationError> {
    let execution =
        ExecutionBudget::with_limits(request.limits.max_steps, request.limits.max_value_bytes);
    residualize_expression_with_ledger(
        expression,
        build_inputs,
        request,
        execution.residual_ledger(),
    )
}

pub(in crate::program) fn residualize_expression_with_ledger(
    expression: &CompiledExpression,
    build_inputs: &ResidualBuildBindings,
    request: ResidualizationRequest,
    ledger: ResidualLedger<'_>,
) -> Result<ResidualizedExpression, ResidualizationError> {
    ledger
        .transaction(|| evaluator::Evaluator::new(expression, build_inputs, request, ledger).run())
}

pub(in crate::program) use closure::residualize_closure_with_ledger;

#[cfg(test)]
mod tests;
