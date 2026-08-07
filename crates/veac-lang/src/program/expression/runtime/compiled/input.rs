use super::value::annotate;
use super::Evaluator;
use crate::program::expression::core::{CompiledFunction, VerifiedCoreProgram};
use crate::program::expression::{ExpressionError, Value};

impl Evaluator<'_> {
    pub(super) fn bind_inputs(
        &self,
        program: &VerifiedCoreProgram,
        current_function: Option<&CompiledFunction>,
    ) -> Result<Vec<Value>, ExpressionError> {
        let core = program.core();
        let result = core
            .inputs
            .iter()
            .map(|input| {
                if matches!(
                    input.identity,
                    crate::program::expression::CoreInputIdentity::Temporal(_)
                ) {
                    return Err(ExpressionError::new(
                        "EXPRESSION_TEMPORAL_INPUT_REQUIRED",
                        "Temporal Core input requires residualization",
                        input.span.clone(),
                    ));
                }
                let crate::program::expression::CoreInputIdentity::Build(identity) =
                    &input.identity
                else {
                    unreachable!("Temporal Core input returned above")
                };
                let value = self
                    .environment
                    .build_value(identity, &input.name)
                    .ok_or_else(|| {
                        ExpressionError::new(
                            "EXPRESSION_UNKNOWN_SYMBOL",
                            format!("unknown symbol `{}`", input.name),
                            input.span.clone(),
                        )
                    })?;
                let expected = core
                    .value_type(input.type_id)
                    .expect("verified Core input has a value type");
                if expected.contains_function_in(program.nominal_types()) == Some(true)
                    && !input.trusted_function
                {
                    super::super::reject_external_function(value, &input.span)?;
                }
                if let Some(contract) = &input.callable {
                    let Value::Closure(closure) = value else {
                        return Err(callable_mismatch(input));
                    };
                    if !contract.matches(closure) {
                        return Err(callable_mismatch(input));
                    }
                }
                let value = value.clone();
                (&value.value_type() == expected)
                    .then_some(value)
                    .ok_or_else(|| {
                        ExpressionError::new(
                            "EXPRESSION_RUNTIME_TYPE",
                            format!("input `{}` expects {expected}", input.name),
                            input.span.clone(),
                        )
                    })
            })
            .collect::<Result<Vec<_>, ExpressionError>>()
            .and_then(|values| {
                for (input, value) in core.inputs.iter().zip(&values) {
                    super::super::validate_value_ref(value, &input.span)?;
                    value
                        .validate_nominal_registry(program.nominal_types())
                        .map_err(|failure| {
                            ExpressionError::new(
                                "EXPRESSION_RUNTIME_CONTRACT",
                                failure.message(),
                                input.span.clone(),
                            )
                        })?;
                    if !input.trusted_function {
                        self.execution.admit_value(value, input.span.clone())?;
                    }
                }
                Ok(values)
            });
        annotate(result, current_function)
    }
}

fn callable_mismatch(input: &crate::program::expression::CoreInput) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_RUNTIME_CONTRACT",
        format!(
            "input `{}` does not match its verified callable contract",
            input.name
        ),
        input.span.clone(),
    )
}

#[cfg(test)]
#[path = "input/tests.rs"]
mod tests;
