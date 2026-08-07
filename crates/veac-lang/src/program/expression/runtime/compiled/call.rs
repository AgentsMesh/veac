use super::Evaluator;
use crate::program::expression::core::{CoreCallTarget, CoreInstruction};
use crate::program::expression::{
    CompiledFunction, ExecutionDefinition, ExecutionFrame, ExpressionCallFrame, ExpressionError,
    Value, MAX_FUNCTION_CALL_DEPTH,
};
use crate::program::DomainOperationRegistry;

impl Evaluator<'_> {
    pub(super) fn call(
        &mut self,
        target: CoreCallTarget,
        values: Vec<Value>,
        instruction: &CoreInstruction,
        call_depth: usize,
        current_function: Option<&CompiledFunction>,
    ) -> Result<Value, ExpressionError> {
        match target {
            CoreCallTarget::Builtin(function) => {
                super::super::function::call_builtin(function, values, instruction.span.clone())
            }
            CoreCallTarget::User(id) => {
                if call_depth >= MAX_FUNCTION_CALL_DEPTH {
                    return Err(ExpressionError::new(
                        "EXPRESSION_CALL_DEPTH_LIMIT",
                        format!("function calls exceed the {MAX_FUNCTION_CALL_DEPTH} depth limit"),
                        instruction.span.clone(),
                    ));
                }
                let function = self.registry.get(id).cloned().ok_or_else(|| {
                    ExpressionError::new(
                        "EXPRESSION_RUNTIME_CONTRACT",
                        format!("compiled call target {id} is unavailable"),
                        instruction.span.clone(),
                    )
                })?;
                super::validate_core_domain_identity(
                    function.body(),
                    DomainOperationRegistry::shared(),
                )?;
                let frame = ExecutionDefinition::function(&function).map(|definition| {
                    ExecutionFrame::new(definition, self.origin(instruction.span.clone()))
                });
                let result = self.program(
                    function.verified_body(),
                    &values,
                    &[],
                    call_depth + 1,
                    Some(function.as_ref()),
                    frame,
                );
                match current_function {
                    Some(caller) => result.map_err(|error| {
                        error.called_from(ExpressionCallFrame::new(
                            caller.name(),
                            caller.origin(),
                            instruction.span.clone(),
                        ))
                    }),
                    None => result,
                }
            }
        }
    }
}
