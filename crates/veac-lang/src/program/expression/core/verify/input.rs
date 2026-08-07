use super::error;
use crate::program::expression::core::CoreProgram;
use crate::program::expression::{ExpressionError, ValueType, ValueTypeKind, MAX_CLOSURE_CAPTURES};
use std::collections::BTreeSet;

pub(super) fn verify_all(
    program: &CoreProgram,
    registry: &crate::program::TypeRegistry,
    trusted_functions: &dyn Fn(&str) -> bool,
) -> Result<(), ExpressionError> {
    let mut identities = BTreeSet::new();
    for input in &program.inputs {
        let value_type = program
            .value_type(input.type_id)
            .ok_or_else(|| error("Core input requires a value type", input.span()))?;
        if !identities.insert(&input.identity) {
            return Err(error("Core input identity must be unique", input.span()));
        }
        let trusted = match &input.identity {
            super::super::CoreInputIdentity::Build(_) => {
                value_type.is_direct_function() && trusted_functions(&input.name)
            }
            super::super::CoreInputIdentity::Temporal(identity) => {
                if value_type != &identity.value_type()
                    || input.trusted_function
                    || input.callable.is_some()
                {
                    return Err(error(
                        "Temporal Core input identity does not match its declared type",
                        input.span(),
                    ));
                }
                false
            }
        };
        if input.trusted_function != trusted {
            return Err(error(
                "Core input function trust does not match its admission",
                input.span(),
            ));
        }
        verify_callable(
            input.callable.as_ref(),
            value_type,
            registry,
            trusted,
            input.span(),
        )?;
    }
    Ok(())
}

fn verify_callable(
    callable: Option<&crate::program::expression::CoreCallableInput>,
    value_type: &ValueType,
    registry: &crate::program::TypeRegistry,
    trusted: bool,
    span: std::ops::Range<usize>,
) -> Result<(), ExpressionError> {
    let direct_function = matches!(value_type.kind(), ValueTypeKind::Function { .. });
    match callable {
        Some(_) if !trusted || !direct_function => Err(error(
            "Core callable input requires an admitted direct function type",
            span,
        )),
        Some(value)
            if value.capture_types.len() > MAX_CLOSURE_CAPTURES
                || value
                    .capture_types
                    .iter()
                    .any(|value| value.contains_function_in(registry) != Some(false)) =>
        {
            Err(error("Core callable input has invalid captures", span))
        }
        None if trusted && direct_function => Err(error(
            "trusted function input requires a callable contract",
            span,
        )),
        _ => Ok(()),
    }
}
