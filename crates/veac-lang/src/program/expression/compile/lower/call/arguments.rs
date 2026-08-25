use super::super::Lowerer;
use crate::program::expression::ast::{CallArgument, Expression};
use crate::program::expression::hir::{TypedCallArgument, TypedDefaultArgument};
use crate::program::expression::{ExpressionError, FunctionId, ValueType};

pub(in crate::program::expression::compile::lower) struct DeclaredParameter<'a> {
    pub name: &'a str,
    pub value_type: ValueType,
    pub default: Option<FunctionId>,
}

pub(in crate::program::expression::compile::lower) struct BoundArguments {
    pub explicit: Vec<TypedCallArgument>,
    pub defaults: Vec<TypedDefaultArgument>,
}

impl Lowerer<'_> {
    pub(in crate::program::expression::compile::lower) fn bind_call_arguments(
        &mut self,
        callable: &str,
        arguments: &[CallArgument],
        parameters: &[DeclaredParameter<'_>],
        slot_offset: usize,
        expression: &Expression,
    ) -> Result<BoundArguments, ExpressionError> {
        let named = arguments
            .iter()
            .filter(|value| value.label.is_some())
            .count();
        if named != 0 && named != arguments.len() {
            return Err(ExpressionError::new(
                "EXPRESSION_CALL_ARGUMENT_MIXED",
                "positional and named arguments cannot be mixed in one call",
                expression.span.clone(),
            ));
        }
        if named == 0 {
            let first_default = parameters
                .iter()
                .position(|parameter| parameter.default.is_some())
                .unwrap_or(parameters.len());
            if arguments.len() < first_default || arguments.len() > parameters.len() {
                return Err(arity(
                    callable,
                    arguments.len(),
                    first_default,
                    parameters.len(),
                    expression,
                ));
            }
            let explicit = arguments
                .iter()
                .zip(parameters)
                .enumerate()
                .map(|(ordinal, (argument, parameter))| {
                    self.typed_argument(argument, parameter, ordinal, ordinal + slot_offset)
                })
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(BoundArguments {
                explicit,
                defaults: missing_defaults(parameters, arguments.len(), slot_offset),
            });
        }
        self.bind_named_arguments(callable, arguments, parameters, slot_offset, expression)
    }

    fn bind_named_arguments(
        &mut self,
        callable: &str,
        arguments: &[CallArgument],
        parameters: &[DeclaredParameter<'_>],
        slot_offset: usize,
        expression: &Expression,
    ) -> Result<BoundArguments, ExpressionError> {
        let mut seen = vec![false; parameters.len()];
        let mut output = Vec::with_capacity(arguments.len());
        for argument in arguments {
            let label = argument.label.as_ref().expect("named call is homogeneous");
            let Some(slot) = parameters.iter().position(|value| value.name == label.name) else {
                return Err(ExpressionError::new(
                    "EXPRESSION_CALL_ARGUMENT_UNKNOWN",
                    format!("{callable} has no parameter named `{}`", label.name),
                    label.span.clone(),
                ));
            };
            if std::mem::replace(&mut seen[slot], true) {
                return Err(ExpressionError::new(
                    "EXPRESSION_CALL_ARGUMENT_DUPLICATE",
                    format!("parameter `{}` is supplied more than once", label.name),
                    label.span.clone(),
                ));
            }
            output.push(self.typed_argument(
                argument,
                &parameters[slot],
                slot,
                slot + slot_offset,
            )?);
        }
        let missing = parameters
            .iter()
            .zip(&seen)
            .filter_map(|(parameter, seen)| {
                (!*seen && parameter.default.is_none()).then_some(parameter.name)
            })
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(ExpressionError::new(
                "EXPRESSION_CALL_ARGUMENT_MISSING",
                format!(
                    "{callable} is missing named parameter(s): {}",
                    missing.join(", ")
                ),
                expression.span.clone(),
            ));
        }
        let defaults = parameters
            .iter()
            .zip(seen)
            .enumerate()
            .filter_map(|(slot, (parameter, seen))| {
                (!seen)
                    .then(|| default_argument(parameter, slot + slot_offset))
                    .flatten()
            })
            .collect();
        Ok(BoundArguments {
            explicit: output,
            defaults,
        })
    }

    fn typed_argument(
        &mut self,
        argument: &CallArgument,
        parameter: &DeclaredParameter<'_>,
        ordinal: usize,
        slot: usize,
    ) -> Result<TypedCallArgument, ExpressionError> {
        let value = self.lower_context(&argument.value, Some(&parameter.value_type))?;
        super::check_value_argument(ordinal, &value, &parameter.value_type, &argument.value)?;
        Ok(TypedCallArgument { slot, value })
    }
}

fn missing_defaults(
    parameters: &[DeclaredParameter<'_>],
    supplied: usize,
    offset: usize,
) -> Vec<TypedDefaultArgument> {
    parameters
        .iter()
        .enumerate()
        .skip(supplied)
        .filter_map(|(slot, parameter)| default_argument(parameter, slot + offset))
        .collect()
}

fn default_argument(
    parameter: &DeclaredParameter<'_>,
    slot: usize,
) -> Option<TypedDefaultArgument> {
    parameter.default.map(|target| TypedDefaultArgument {
        slot,
        target,
        value_type: parameter.value_type.clone(),
    })
}

fn arity(
    callable: &str,
    actual: usize,
    minimum: usize,
    maximum: usize,
    expression: &Expression,
) -> ExpressionError {
    ExpressionError::new(
        "EXPRESSION_CALL_ARITY",
        format!("{callable} expects {minimum}..={maximum} arguments, found {actual}"),
        expression.span.clone(),
    )
}

pub(in crate::program::expression::compile::lower) fn reject_named(
    callable: &str,
    arguments: &[CallArgument],
    expression: &Expression,
) -> Result<(), ExpressionError> {
    if arguments.iter().any(|argument| argument.label.is_some()) {
        return Err(ExpressionError::new(
            "EXPRESSION_CALL_NAMED_UNSUPPORTED",
            format!("{callable} does not publish stable parameter names"),
            expression.span.clone(),
        ));
    }
    Ok(())
}
