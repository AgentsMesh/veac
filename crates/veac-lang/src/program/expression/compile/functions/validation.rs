use std::collections::BTreeSet;

use crate::program::expression::{
    BuiltinFunction, CollectionOperation, ExpressionContext, ExpressionError, FunctionDefinition,
    MAX_FUNCTION_PARAMETERS,
};

pub(crate) fn validate_definitions(
    context: &ExpressionContext,
    definitions: &[FunctionDefinition],
) -> Result<(), ExpressionError> {
    super::super::known_type::context(context)?;
    let mut names = BTreeSet::new();
    for definition in definitions {
        let span = 0..definition.body.len().min(1);
        if !crate::name::is_qualified_name(&definition.name)
            || BuiltinFunction::parse(&definition.name).is_some()
            || CollectionOperation::parse(&definition.name).is_some()
            || context.domain().lookup_function(&definition.name).is_some()
        {
            return Err(ExpressionError::new(
                "EXPRESSION_FUNCTION_NAME",
                format!("invalid or reserved function name `{}`", definition.name),
                span,
            )
            .in_function(&definition.name));
        }
        if context.functions().lookup(&definition.name).is_some() || !names.insert(&definition.name)
        {
            return Err(ExpressionError::new(
                "EXPRESSION_DUPLICATE_FUNCTION",
                format!("function `{}` is already defined", definition.name),
                span,
            )
            .in_function(&definition.name));
        }
        validate_parameters(definition)?;
        super::super::known_type::signature(
            definition.parameters.iter().map(|value| &value.value_type),
            &definition.return_type,
            context.types(),
        )
        .map_err(|error| error.in_function(&definition.name))?;
    }
    Ok(())
}

fn validate_parameters(definition: &FunctionDefinition) -> Result<(), ExpressionError> {
    if definition.parameters.len() > MAX_FUNCTION_PARAMETERS {
        return Err(ExpressionError::new(
            "EXPRESSION_FUNCTION_PARAMETER_LIMIT",
            format!(
                "function `{}` exceeds the {MAX_FUNCTION_PARAMETERS} parameter limit",
                definition.name
            ),
            0..definition.body.len().min(1),
        )
        .in_function(&definition.name));
    }
    let mut names = BTreeSet::new();
    let mut found_default = false;
    for parameter in &definition.parameters {
        if !crate::name::is_name(&parameter.name) {
            return Err(ExpressionError::new(
                "EXPRESSION_PARAMETER_NAME",
                format!("invalid parameter name `{}`", parameter.name),
                0..definition.body.len().min(1),
            )
            .in_function(&definition.name));
        }
        if !names.insert(&parameter.name) {
            return Err(ExpressionError::new(
                "EXPRESSION_DUPLICATE_PARAMETER",
                format!(
                    "function `{}` declares parameter `{}` more than once",
                    definition.name, parameter.name
                ),
                0..definition.body.len().min(1),
            )
            .in_function(&definition.name));
        }
        if parameter.has_default() {
            found_default = true;
        } else if found_default {
            return Err(ExpressionError::new(
                "EXPRESSION_REQUIRED_PARAMETER_AFTER_DEFAULT",
                "required parameters cannot follow a parameter with a default",
                0..definition.body.len().min(1),
            )
            .in_function(&definition.name));
        }
    }
    Ok(())
}
