use super::slot::{public_at, RuntimeValue};
use super::value::annotate;
use super::Evaluator;
use crate::program::expression::core::{
    BlockId, CompiledFunction, CoreBlock, CoreTerminator, VerifiedCoreProgram,
};
use crate::program::expression::{ExpressionError, Value};

pub(super) enum Flow {
    Return(Value),
    Continue(BlockId),
}

impl Evaluator<'_> {
    pub(super) fn terminator(
        &mut self,
        block: &CoreBlock,
        values: &mut [Option<RuntimeValue>],
        program: &VerifiedCoreProgram,
        call_depth: usize,
        current_function: Option<&CompiledFunction>,
    ) -> Result<Flow, ExpressionError> {
        let result = match &block.terminator {
            CoreTerminator::Return { value, span } => {
                public_at(values, *value, span).map(Flow::Return)
            }
            CoreTerminator::Jump {
                target,
                arguments,
                span,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|id| public_at(values, *id, span))
                    .collect::<Result<Vec<_>, _>>()?;
                let target_block =
                    &program.core().blocks[target.index().expect("verified block target")];
                for (parameter, value) in target_block.parameters.iter().zip(arguments) {
                    values[parameter.id.index().expect("verified parameter ID")] =
                        Some(RuntimeValue::Public(value));
                }
                Ok(Flow::Continue(*target))
            }
            CoreTerminator::Branch {
                condition,
                then_target,
                else_target,
                span,
            } => {
                self.enter(span.clone())?;
                match public_at(values, *condition, span)? {
                    Value::Bool(true) => Ok(Flow::Continue(*then_target)),
                    Value::Bool(false) => Ok(Flow::Continue(*else_target)),
                    _ => Err(ExpressionError::new(
                        "EXPRESSION_RUNTIME_TYPE",
                        "Core branch condition is not bool",
                        span.clone(),
                    )),
                }
            }
            CoreTerminator::Match {
                scrutinee,
                arms,
                span,
            } => {
                self.enter(span.clone())?;
                let Value::Enum(value) = public_at(values, *scrutinee, span)? else {
                    return annotate(
                        Err(ExpressionError::new(
                            "EXPRESSION_RUNTIME_TYPE",
                            "Core match scrutinee is not an enum",
                            span.clone(),
                        )),
                        current_function,
                    );
                };
                let arm = arms
                    .get(value.variant().index())
                    .filter(|arm| arm.variant() == value.variant())
                    .ok_or_else(|| {
                        ExpressionError::new(
                            "EXPRESSION_RUNTIME_CONTRACT",
                            "Core match has no verified arm for the selected variant",
                            span.clone(),
                        )
                    })?;
                let target =
                    &program.core().blocks[arm.target().index().expect("verified match target")];
                for (parameter, field) in target.parameters.iter().zip(value.fields()) {
                    values[parameter.id.index().expect("verified parameter ID")] =
                        Some(RuntimeValue::Public(field.clone()));
                }
                Ok(Flow::Continue(arm.target()))
            }
            CoreTerminator::ForEach(value) => self.for_each(value, program, values, call_depth),
        };
        annotate(result, current_function)
    }
}
