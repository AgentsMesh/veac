use super::super::super::evaluator::Evaluator;
use super::super::super::{ResidualRuntimeValue, ResidualizationError};
use crate::program::expression::{Effect, FunctionId, MAX_FUNCTION_CALL_DEPTH};

impl Evaluator<'_> {
    pub(super) fn user(
        &mut self,
        id: FunctionId,
        arguments: Vec<ResidualRuntimeValue>,
        span: std::ops::Range<usize>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        if self.call_depth >= MAX_FUNCTION_CALL_DEPTH {
            return Err(ResidualizationError::new(
                "RESIDUAL_CALL_DEPTH_LIMIT",
                format!("function calls exceed the {MAX_FUNCTION_CALL_DEPTH} depth limit"),
                span,
            ));
        }
        let function = self
            .expression
            .registry_arc()
            .get(id)
            .cloned()
            .ok_or_else(|| {
                ResidualizationError::new(
                    "RESIDUAL_CORE_CONTRACT",
                    format!("compiled call target {id} is unavailable"),
                    span.clone(),
                )
            })?;
        if function.summary().effect() != Effect::Pure {
            return Err(ResidualizationError::new(
                "RESIDUAL_CALL_EFFECT",
                "Temporal function calls must be Pure",
                span,
            ));
        }
        self.call_program(function.body().clone(), arguments, Vec::new())
    }

    pub(in crate::program::expression::residual) fn call_program(
        &mut self,
        core: crate::program::expression::CoreProgram,
        parameters: Vec<ResidualRuntimeValue>,
        captures: Vec<ResidualRuntimeValue>,
    ) -> Result<ResidualRuntimeValue, ResidualizationError> {
        let previous_core = std::mem::replace(&mut self.current_core, core);
        let previous_parameters = std::mem::replace(&mut self.parameters, parameters);
        let previous_captures = std::mem::replace(&mut self.captures, captures);
        self.call_depth += 1;
        let slots = vec![None; self.core().value_count()];
        let result = self.block(self.core().entry(), slots);
        self.call_depth -= 1;
        self.parameters = previous_parameters;
        self.captures = previous_captures;
        self.current_core = previous_core;
        result
    }
}
